mod backend;

use crate::{
	error::ConductorError,
	id::{InstanceId, MetronomeId, SoundId},
	project::Project,
};
use backend::{Backend, Command};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Producer, RingBuffer};
use std::error::Error;

pub struct InstanceSettings {
	pub volume: f32,
	pub pitch: f32,
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self {
			volume: 1.0,
			pitch: 1.0,
		}
	}
}

pub struct AudioManagerSettings {
	pub num_commands: usize,
	pub num_instances: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_instances: 100,
		}
	}
}

pub struct AudioManager {
	command_producer: Producer<Command>,
	_stream: Stream,
}

impl AudioManager {
	pub fn new(project: Project, settings: AudioManagerSettings) -> Result<Self, Box<dyn Error>> {
		let host = cpal::default_host();
		let device = host.default_output_device().unwrap();
		let mut supported_configs_range = device.supported_output_configs().unwrap();
		let supported_config = supported_configs_range
			.next()
			.unwrap()
			.with_max_sample_rate();
		let config = supported_config.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let mut backend = Backend::new(sample_rate, project, command_consumer, settings);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					frame[0] = out.left;
					frame[1] = out.right;
				}
			},
			move |_| {},
		)?;
		stream.play()?;
		Ok(Self {
			command_producer,
			_stream: stream,
		})
	}

	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: InstanceSettings,
	) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self
			.command_producer
			.push(Command::PlaySound(sound_id, instance_id, settings))
		{
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	pub fn start_metronome(&mut self, id: MetronomeId) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self.command_producer.push(Command::StartMetronome(id)) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	pub fn pause_metronome(&mut self, id: MetronomeId) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self.command_producer.push(Command::PauseMetronome(id)) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	pub fn stop_metronome(&mut self, id: MetronomeId) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self.command_producer.push(Command::StopMetronome(id)) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}
}
