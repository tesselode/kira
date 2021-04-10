mod backend;
pub(crate) mod command;
pub mod error;

use std::sync::Arc;

use basedrop::{Collector, Handle, Shared};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Producer, RingBuffer};

use crate::{
	error::CommandQueueFullError,
	metronome::{handle::MetronomeHandle, settings::MetronomeSettings, Metronome, MetronomeState},
	sequence::{instance::SequenceInstance, Sequence},
	sound::{
		instance::{handle::InstanceHandle, Instance, InstanceController},
		Sound,
	},
};

use self::{backend::Backend, command::Command, error::SetupError};

pub struct AudioManagerSettings {
	num_commands: usize,
	num_instances: usize,
	num_metronomes: usize,
	num_sequences: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_instances: 100,
			num_metronomes: 10,
			num_sequences: 25,
		}
	}
}

pub struct AudioManager {
	_stream: Stream,
	command_producer: Producer<Command>,
	collector: Collector,
	collector_handle: Handle,
}

impl AudioManager {
	pub fn new(settings: AudioManagerSettings) -> Result<Self, SetupError> {
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let collector = Collector::new();
		let collector_handle = collector.handle();
		Ok(Self {
			_stream: {
				let host = cpal::default_host();
				let device = host
					.default_output_device()
					.ok_or(SetupError::NoDefaultOutputDevice)?;
				let config = device.default_output_config()?.config();
				let sample_rate = config.sample_rate.0;
				let channels = config.channels;
				let mut backend = Backend::new(sample_rate, command_consumer, settings);
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
		})
	}

	pub fn play(&mut self, sound: Arc<Sound>) -> Result<InstanceHandle, CommandQueueFullError> {
		let controller = Shared::new(&self.collector_handle, InstanceController::new());
		let instance = Instance::new(sound, controller.clone());
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

	pub fn start_sequence(
		&mut self,
		sequence: Sequence<usize>,
	) -> Result<(), CommandQueueFullError> {
		let (event_producer, event_consumer) = RingBuffer::new(1).split();
		let instance = SequenceInstance::new(sequence, &self.collector_handle, event_producer);
		self.command_producer
			.push(Command::StartSequenceInstance(instance))
			.map_err(|_| CommandQueueFullError)?;
		Ok(())
	}
}
