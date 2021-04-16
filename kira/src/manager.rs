mod backend;
pub(crate) mod command;
pub mod error;

use std::{hash::Hash, path::Path};

use basedrop::{Collector, Handle, Owned, Shared};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Producer, RingBuffer};

use crate::{
	error::CommandQueueFullError,
	metronome::{handle::MetronomeHandle, settings::MetronomeSettings, Metronome, MetronomeState},
	mixer::track::{handle::TrackHandle, settings::SubTrackSettings, Track, TrackInput},
	parameter::Parameter,
	sequence::{
		instance::{handle::SequenceInstanceHandle, SequenceInstance},
		Sequence,
	},
	sound::{
		data::{static_sound::StaticSoundData, SoundData},
		handle::SoundHandle,
		instance::{handle::InstanceHandle, settings::InstanceSettings, Instance},
		Sound,
	},
};

#[cfg(not(feature = "benchmarking"))]
use backend::Backend;
#[cfg(feature = "benchmarking")]
pub use backend::Backend;

use self::{
	command::Command,
	error::{LoadSoundError, SetupError, StartSequenceError},
};

pub struct AudioManagerSettings {
	pub num_commands: usize,
	pub num_sounds: usize,
	pub num_instances: usize,
	pub num_metronomes: usize,
	pub num_sequences: usize,
	pub num_parameters: usize,
	pub num_sub_tracks: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_sounds: 100,
			num_instances: 100,
			num_metronomes: 10,
			num_sequences: 25,
			num_parameters: 100,
			num_sub_tracks: 25,
		}
	}
}

pub struct AudioManager {
	_stream: Option<Stream>,
	sample_rate: u32,
	command_producer: Producer<Command>,
	collector: Collector,
	collector_handle: Handle,
	main_track_input: TrackInput,
}

impl AudioManager {
	pub fn new(settings: AudioManagerSettings) -> Result<Self, SetupError> {
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let collector = Collector::new();
		let collector_handle = collector.handle();
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(SetupError::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let mut backend = Backend::new(sample_rate, command_consumer, collector.handle(), settings);
		let main_track_input = backend.main_track_input();
		Ok(Self {
			_stream: Some({
				let stream = device.build_output_stream(
					&config,
					move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
						for frame in data.chunks_exact_mut(channels as usize) {
							let out = backend.process();
							if channels == 1 {
								frame[0] = (out.left + out.right) / 2.0;
							} else {
								frame[0] = out.left;
								frame[1] = out.right;
							}
						}
					},
					move |_| {},
				)?;
				stream.play()?;
				stream
			}),
			sample_rate,
			command_producer,
			collector,
			collector_handle,
			main_track_input,
		})
	}

	#[cfg(any(feature = "benchmarking", test))]
	/// Creates an [`AudioManager`] and [`Backend`] without sending
	/// the backend to another thread.
	///
	/// This is useful for updating the backend manually for
	/// benchmarking.
	pub fn new_without_audio_thread(settings: AudioManagerSettings) -> (Self, Backend) {
		const SAMPLE_RATE: u32 = 48000;
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let collector = Collector::new();
		let collector_handle = collector.handle();
		let backend = Backend::new(SAMPLE_RATE, command_consumer, collector.handle(), settings);
		let main_track_input = backend.main_track_input();
		let audio_manager = Self {
			_stream: None,
			sample_rate: SAMPLE_RATE,
			command_producer,
			collector,
			collector_handle,
			main_track_input,
		};
		(audio_manager, backend)
	}

	pub fn add_sound(
		&mut self,
		data: impl SoundData + 'static,
	) -> Result<SoundHandle, CommandQueueFullError> {
		let sound = Shared::new(&self.collector_handle, Sound::new(data));
		let handle = SoundHandle::new(sound.clone());
		self.command_producer
			.push(Command::AddSound(sound))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}

	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn load_sound(&mut self, path: impl AsRef<Path>) -> Result<SoundHandle, LoadSoundError> {
		let data = StaticSoundData::from_file(path)?;
		let handle = self
			.add_sound(data)
			.map_err(|_| LoadSoundError::CommandQueueFullError)?;
		Ok(handle)
	}

	pub fn play(
		&mut self,
		sound: &SoundHandle,
		settings: InstanceSettings,
	) -> Result<InstanceHandle, CommandQueueFullError> {
		let instance = Shared::new(
			&self.collector_handle,
			Instance::new(
				sound.sound().clone(),
				settings.track.unwrap_or(self.main_track_input.clone()),
			),
		);
		let handle = InstanceHandle::new(instance.clone());
		self.command_producer
			.push(Command::StartInstance(instance))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}

	pub fn add_metronome(
		&mut self,
		settings: MetronomeSettings,
	) -> Result<MetronomeHandle, CommandQueueFullError> {
		let (interval_event_producer, interval_event_consumer) =
			RingBuffer::new(settings.interval_events_to_emit.len()).split();
		let state = Shared::new(&self.collector_handle, MetronomeState::new(settings.tempo));
		let metronome = Owned::new(
			&self.collector_handle,
			Metronome::new(
				state.clone(),
				settings.interval_events_to_emit,
				interval_event_producer,
			),
		);
		let handle = MetronomeHandle::new(state.clone(), interval_event_consumer);
		self.command_producer
			.push(Command::AddMetronome(metronome))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}

	pub fn start_sequence<'a, Event: Clone + Eq + Hash>(
		&mut self,
		sequence: Sequence<Event>,
		metronome: impl Into<Option<&'a MetronomeHandle>>,
	) -> Result<SequenceInstanceHandle<Event>, StartSequenceError> {
		sequence.validate()?;
		let (raw_sequence, events) = sequence.create_raw_sequence();
		let (event_producer, event_consumer) = RingBuffer::new(events.len()).split();
		let instance = Owned::new(
			&self.collector_handle,
			SequenceInstance::new(
				raw_sequence,
				metronome.into().map(|handle| handle.state()),
				event_producer,
			),
		);
		let handle = SequenceInstanceHandle::new(events, event_consumer);
		self.command_producer
			.push(Command::StartSequenceInstance(instance))
			.map_err(|_| StartSequenceError::CommandQueueFullError)?;
		Ok(handle)
	}

	pub fn add_parameter(&mut self, value: f64) -> Result<Parameter, CommandQueueFullError> {
		let parameter = Parameter::new(value, &self.collector_handle);
		self.command_producer
			.push(Command::AddParameter(parameter.clone()))
			.map_err(|_| CommandQueueFullError)?;
		Ok(parameter)
	}

	pub fn add_sub_track(
		&mut self,
		settings: SubTrackSettings,
	) -> Result<TrackHandle, CommandQueueFullError> {
		let (effect_slot_producer, effect_slot_consumer) =
			RingBuffer::new(settings.num_effects).split();
		let sub_track = Track::new(
			&self.collector_handle,
			settings.routes.to_vec(self.main_track_input.clone()),
			settings.volume,
			settings.num_effects,
			effect_slot_consumer,
		);
		let handle = TrackHandle::new(
			sub_track.input().clone(),
			effect_slot_producer,
			self.collector.handle(),
			self.sample_rate,
		);
		self.command_producer
			.push(Command::AddSubTrack(Owned::new(
				&self.collector_handle,
				sub_track,
			)))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}
}
