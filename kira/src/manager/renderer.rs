pub mod context;

use std::sync::Arc;

use ringbuf::Consumer;

use crate::frame::Frame;

use self::context::Context;

use super::{command::Command, resources::Resources};

pub struct Renderer {
	context: Arc<Context>,
	resources: Resources,
	command_consumer: Consumer<Command>,
}

impl Renderer {
	pub(super) fn new(
		context: Arc<Context>,
		resources: Resources,
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			context,
			resources,
			command_consumer,
		}
	}

	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
		self.resources.instances.on_start_processing();
		self.resources.parameters.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Instance(command) => self.resources.instances.run_command(command),
				Command::Parameter(command) => self.resources.parameters.run_command(command),
				Command::Mixer(command) => self.resources.mixer.run_command(command),
				Command::Clock(command) => self.resources.clocks.run_command(command),
			}
		}
	}

	pub fn process(&mut self) -> Frame {
		self.resources
			.clocks
			.update(self.context.dt, &self.resources.parameters);
		self.resources
			.parameters
			.update(self.context.dt, &self.resources.clocks);
		self.resources.instances.process(
			self.context.dt,
			&self.resources.sounds,
			&self.resources.parameters,
			&self.resources.clocks,
			&mut self.resources.mixer,
		);
		let out = self
			.resources
			.mixer
			.process(self.context.dt, &self.resources.parameters);
		out
	}
}
