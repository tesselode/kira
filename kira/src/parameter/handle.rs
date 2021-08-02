use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, Command, ParameterCommand},
};

use super::{ParameterId, ParameterShared, Tween};

pub struct ParameterHandle {
	pub(crate) id: ParameterId,
	pub(crate) shared: Arc<ParameterShared>,
	pub(crate) command_producer: CommandProducer,
}

impl ParameterHandle {
	pub fn id(&self) -> ParameterId {
		self.id
	}

	pub fn value(&self) -> f64 {
		self.shared.value()
	}

	pub fn paused(&self) -> bool {
		self.shared.paused()
	}

	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Pause(self.id)))
	}

	pub fn resume(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Resume(self.id)))
	}

	pub fn set(&mut self, target: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Set {
				id: self.id,
				target,
				tween,
			}))
	}
}

impl Drop for ParameterHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
