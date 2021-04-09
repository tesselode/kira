use ringbuf::Producer;

use crate::error::CommandQueueFullError;

use super::Command;

pub struct InstanceHandle {
	command_producer: Producer<Command>,
}

impl InstanceHandle {
	pub(crate) fn new(command_producer: Producer<Command>) -> Self {
		Self { command_producer }
	}

	pub fn pause(&mut self) -> Result<(), CommandQueueFullError> {
		self.command_producer
			.push(Command::Pause)
			.map_err(|_| CommandQueueFullError)
	}

	pub fn resume(&mut self) -> Result<(), CommandQueueFullError> {
		self.command_producer
			.push(Command::Resume)
			.map_err(|_| CommandQueueFullError)
	}

	pub fn stop(&mut self) -> Result<(), CommandQueueFullError> {
		self.command_producer
			.push(Command::Stop)
			.map_err(|_| CommandQueueFullError)
	}
}
