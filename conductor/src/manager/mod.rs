mod backend;

use crate::{error::ConductorError, project::Project};
use backend::{Backend, Command};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Producer, RingBuffer};
use std::error::Error;

pub struct AudioManagerSettings {
	pub num_commands: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self { num_commands: 100 }
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
		let mut backend = Backend::new(sample_rate, project, command_consumer);
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

	pub fn test(&mut self) -> Result<(), ConductorError> {
		match self.command_producer.push(Command::Test) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}
}
