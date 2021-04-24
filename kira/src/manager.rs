mod backend;
pub(crate) mod command;
pub mod error;

use std::{
	hash::Hash,
	io::{stderr, Write},
	path::Path,
};

use basedrop::{Collector, Owned, Shared};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use instant::{Duration, Instant};
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
		settings::SoundSettings,
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

const DROP_CLEANUP_TIMEOUT: Duration = Duration::from_millis(1000);

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
	stream: Option<Stream>,
	sample_rate: u32,
	command_producer: Producer<Command>,
	collector: Option<Collector>,
	main_track_input: Option<TrackInput>,
}

impl AudioManager {
	pub fn new(settings: AudioManagerSettings) -> Result<Self, SetupError> {
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let collector = Collector::new();
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
			stream: Some({
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
			collector: Some(collector),
			main_track_input: Some(main_track_input),
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
		let backend = Backend::new(SAMPLE_RATE, command_consumer, collector.handle(), settings);
		let main_track_input = backend.main_track_input();
		let audio_manager = Self {
			stream: None,
			sample_rate: SAMPLE_RATE,
			command_producer,
			collector: Some(collector),
			main_track_input: Some(main_track_input),
		};
		(audio_manager, backend)
	}

	fn collector(&self) -> &Collector {
		self.collector.as_ref().unwrap()
	}

	fn collector_mut(&mut self) -> &mut Collector {
		self.collector.as_mut().unwrap()
	}

	pub fn add_sound(
		&mut self,
		data: impl SoundData + 'static,
		settings: SoundSettings,
	) -> Result<SoundHandle, CommandQueueFullError> {
		let sound = Shared::new(
			&self.collector().handle(),
			Sound::new(
				data,
				settings.loop_start,
				settings.semantic_duration,
				settings.cooldown,
			),
		);
		let handle = SoundHandle::new(sound.clone());
		self.command_producer
			.push(Command::AddSound(sound))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}

	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn load_sound(
		&mut self,
		path: impl AsRef<Path>,
		settings: SoundSettings,
	) -> Result<SoundHandle, LoadSoundError> {
		let data = StaticSoundData::from_file(path)?;
		let handle = self
			.add_sound(data, settings)
			.map_err(|_| LoadSoundError::CommandQueueFullError)?;
		Ok(handle)
	}

	pub fn play(
		&mut self,
		sound: &SoundHandle,
		settings: InstanceSettings,
	) -> Result<InstanceHandle, CommandQueueFullError> {
		let instance = Shared::new(
			&self.collector().handle(),
			Instance::new(
				sound.sound().clone(),
				settings.into_internal(
					sound.sound(),
					self.main_track_input.as_ref().unwrap().clone(),
				),
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
		let state = Shared::new(
			&self.collector().handle(),
			MetronomeState::new(settings.tempo),
		);
		let metronome = Owned::new(
			&self.collector().handle(),
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
			&self.collector().handle(),
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
		let parameter = Parameter::new(value, &self.collector().handle());
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
			&self.collector().handle(),
			settings
				.routes
				.to_vec(self.main_track_input.as_ref().unwrap().clone()),
			settings.volume,
			settings.num_effects,
			effect_slot_consumer,
		);
		let handle = TrackHandle::new(
			sub_track.input().clone(),
			effect_slot_producer,
			self.collector().handle(),
			self.sample_rate,
		);
		self.command_producer
			.push(Command::AddSubTrack(Owned::new(
				&self.collector().handle(),
				sub_track,
			)))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}

	pub fn free_unused_resources(&mut self) {
		self.collector_mut().collect();
	}

	pub fn shutdown(mut self) -> Result<(), Self> {
		self.stream.take();
		self.main_track_input.take();
		self.free_unused_resources();
		if let Err(collector) = self.collector.take().unwrap().try_cleanup() {
			self.collector = Some(collector);
			return Err(self);
		}
		Ok(())
	}
}

#[cfg(not(feature = "benchmarking"))]
impl Drop for AudioManager {
	fn drop(&mut self) {
		self.stream.take();
		self.main_track_input.take();
		let start_time = Instant::now();
		while let Some(mut collector) = self.collector.take() {
			collector.collect();
			match collector.try_cleanup() {
				Ok(_) => {
					break;
				}
				Err(collector) => {
					self.collector = Some(collector);
				}
			}
			if Instant::now() - start_time > DROP_CLEANUP_TIMEOUT {
				writeln!(
					stderr(),
					"Kira failed to cleanup resources after {} milliseconds, giving up",
					DROP_CLEANUP_TIMEOUT.as_millis()
				)
				.ok();
				break;
			}
		}
	}
}
