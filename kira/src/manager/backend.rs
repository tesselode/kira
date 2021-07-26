use ringbuf::Consumer;

use crate::frame::Frame;

use super::{command::Command, resources::Resources};

pub(super) struct Backend {
	dt: f64,
	resources: Resources,
	command_consumer: Consumer<Command>,
}

impl Backend {
	pub fn new(
		sample_rate: u32,
		resources: Resources,
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			resources,
			command_consumer,
		}
	}

	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
		self.resources.instances.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Instance(command) => self.resources.instances.run_command(command),
			}
		}
	}

	pub fn process(&mut self) -> Frame {
		self.resources
			.instances
			.process(self.dt, &self.resources.sounds)
	}
}
