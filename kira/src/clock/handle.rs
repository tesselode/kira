use std::sync::Arc;

use crate::{
	error::CommandError,
	manager::command::{producer::CommandProducer, ClockCommand, Command},
	value::Value,
};

use super::{ClockId, ClockShared};

pub struct ClockHandle {
	pub(crate) id: ClockId,
	pub(crate) shared: Arc<ClockShared>,
	pub(crate) command_producer: CommandProducer,
}

impl ClockHandle {
	pub fn id(&self) -> ClockId {
		self.id
	}

	pub fn ticking(&self) -> bool {
		self.shared.ticking()
	}

	pub fn time(&self) -> u64 {
		self.shared.time()
	}

	pub fn set_interval(&mut self, interval: impl Into<Value>) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::SetInterval(
				self.id,
				interval.into(),
			)))
	}

	pub fn start(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::Start(self.id)))
	}

	pub fn pause(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::Pause(self.id)))
	}

	pub fn stop(&mut self) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Clock(ClockCommand::Stop(self.id)))
	}
}

impl Drop for ClockHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&ClockHandle> for ClockId {
	fn from(handle: &ClockHandle) -> Self {
		handle.id()
	}
}
