use std::{cell::RefCell, rc::Rc};

use ringbuf::Producer;

use crate::{command::Command, AudioError, AudioResult};

#[derive(Clone)]
pub(crate) struct CommandProducer {
	producer: Rc<RefCell<Producer<Command>>>,
}

impl CommandProducer {
	pub fn new(producer: Producer<Command>) -> Self {
		Self {
			producer: Rc::new(RefCell::new(producer)),
		}
	}

	pub fn push(&mut self, command: Command) -> AudioResult<()> {
		self.producer
			.try_borrow_mut()
			.map_err(|_| AudioError::CommandQueueBorrowed)?
			.push(command)
			.map_err(|_| AudioError::CommandQueueFull)
	}
}
