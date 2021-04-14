mod backend;
pub(crate) mod command;
pub mod error;

use std::{hash::Hash, sync::Arc};

use basedrop::{Collector, Handle, Shared};
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
		instance::{
			handle::InstanceHandle, settings::InstanceSettings, Instance, InstanceController,
		},
		Sound,
	},
};

use self::{backend::Backend, command::Command, error::SetupError};

pub struct AudioManagerSettings {
	num_commands: usize,
	num_instances: usize,
	num_metronomes: usize,
	num_sequences: usize,
	num_parameters: usize,
	num_sub_tracks: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_instances: 100,
			num_metronomes: 10,
			num_sequences: 25,
			num_parameters: 100,
			num_sub_tracks: 25,
		}
	}
}

pub struct AudioManager {
	_stream: Stream,
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
		let mut backend = Backend::new(sample_rate, command_consumer, &collector_handle, settings);
		let main_track_input = backend.main_track_input();
		Ok(Self {
			_stream: {
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
			},
			command_producer,
			collector,
			collector_handle,
			main_track_input,
		})
	}

	pub fn play(
		&mut self,
		sound: Arc<Sound>,
		settings: InstanceSettings,
	) -> Result<InstanceHandle, CommandQueueFullError> {
		let controller = Shared::new(&self.collector_handle, InstanceController::new());
		let instance = Instance::new(
			sound,
			controller.clone(),
			settings.track.unwrap_or(self.main_track_input.clone()),
		);
		let handle = InstanceHandle::new(controller.clone());
		self.command_producer
			.push(Command::StartInstance { instance })
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
		let metronome = Metronome::new(
			state.clone(),
			settings.interval_events_to_emit,
			interval_event_producer,
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
	) -> Result<SequenceInstanceHandle<Event>, CommandQueueFullError> {
		let (raw_sequence, events) = sequence.create_raw_sequence();
		let (event_producer, event_consumer) = RingBuffer::new(events.len()).split();
		let instance = SequenceInstance::new(
			raw_sequence,
			metronome.into().map(|handle| handle.state()),
			&self.collector_handle,
			event_producer,
		);
		let handle = SequenceInstanceHandle::new(events, event_consumer);
		self.command_producer
			.push(Command::StartSequenceInstance(instance))
			.map_err(|_| CommandQueueFullError)?;
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
		let sub_track = Track::new(
			&self.collector_handle,
			Some(settings.parent.unwrap_or(self.main_track_input.clone())),
			settings.volume,
		);
		let handle = TrackHandle::new(sub_track.input().clone());
		self.command_producer
			.push(Command::AddSubTrack(sub_track))
			.map_err(|_| CommandQueueFullError)?;
		Ok(handle)
	}
}