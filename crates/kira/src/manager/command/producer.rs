use std::sync::{Arc, Mutex};

use ringbuf::Producer;

use crate::error::CommandError;

use super::Command;

#[derive(Clone)]
pub(crate) struct CommandProducer(Arc<Mutex<Producer<Command>>>);

impl CommandProducer {
	pub fn new(raw_producer: Producer<Command>) -> Self {
		Self(Arc::new(Mutex::new(raw_producer)))
	}

	pub fn push(&mut self, command: Command) -> Result<(), CommandError> {
		self.0
			.lock()
			.map_err(|_| CommandError::MutexPoisoned)?
			.push(command)
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
