pub mod context;

use std::sync::{atomic::Ordering, Arc};

use ringbuf::Consumer;

use crate::frame::Frame;

use self::context::Context;

use super::{command::Command, resources::Resources};

pub(super) struct Backend {
	context: Arc<Context>,
	sample_count: u64,
	resources: Resources,
	command_consumer: Consumer<Command>,
}

impl Backend {
	pub fn new(
		context: Arc<Context>,
		resources: Resources,
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			context,
			sample_count: 0,
			resources,
			command_consumer,
		}
	}

	pub fn on_start_processing(&mut self) {
		self.context
			.sample_count
			.store(self.sample_count, Ordering::SeqCst);

		self.resources.sounds.on_start_processing();
		self.resources.instances.on_start_processing();
		self.resources.parameters.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Instance(command) => {
					self.resources.instances.run_command(command, &self.context)
				}
				Command::Parameter(command) => self
					.resources
					.parameters
					.run_command(command, &self.context),
			}
		}
	}

	pub fn process(&mut self) -> Frame {
		self.resources.parameters.update(self.context.dt);
		let out = self.resources.instances.process(
			self.sample_count,
			self.context.dt,
			&self.resources.sounds,
			&self.resources.parameters,
		);
		self.sample_count += 1;
		out
	}
}
