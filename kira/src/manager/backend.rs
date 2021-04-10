use ringbuf::Consumer;

use crate::{
	sound::instance::{Instance, InstancePlaybackState},
	Frame,
};

use super::{command::Command, AudioManagerSettings};

pub struct Backend {
	sample_rate: u32,
	dt: f64,
	command_consumer: Consumer<Command>,
	instances: Vec<Instance>,
}

impl Backend {
	pub(crate) fn new(
		sample_rate: u32,
		command_consumer: Consumer<Command>,
		settings: AudioManagerSettings,
	) -> Self {
		Self {
			sample_rate,
			dt: 1.0 / sample_rate as f64,
			command_consumer,
			instances: Vec::with_capacity(settings.num_instances),
		}
	}

	pub fn process(&mut self) -> Frame {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::PlaySound { instance } => {
					if self.instances.len() < self.instances.capacity() {
						self.instances.push(instance);
					}
				}
			}
		}

		let dt = self.dt;
		let output = self
			.instances
			.iter_mut()
			.fold(Frame::from_mono(0.0), |previous, instance| {
				previous + instance.process(dt)
			});
		self.instances
			.retain(|instance| instance.state() != InstancePlaybackState::Stopped);
		output
	}
}
