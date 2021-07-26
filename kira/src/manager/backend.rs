use ringbuf::Consumer;

use crate::frame::Frame;

use super::{
	command::Command,
	resources::{Resources, UnusedResourceProducers},
};

pub(super) struct Backend {
	sample_rate: u32,
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
			sample_rate,
			resources,
			command_consumer,
		}
	}

	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
			}
		}
	}

	pub fn process(&mut self) -> Frame {
		Frame::from_mono(0.0)
	}
}
