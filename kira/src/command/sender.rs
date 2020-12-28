use std::{cell::RefCell, rc::Rc};

use ringbuf::Sender;

use crate::{command::Command, AudioError, AudioResult};

#[derive(Clone)]
pub(crate) struct CommandSender {
	sender: Rc<RefCell<Sender<Command>>>,
}

impl CommandSender {
	pub fn new(sender: Sender<Command>) -> Self {
		Self {
			sender: Rc::new(RefCell::new(sender)),
		}
	}

	pub fn push(&mut self, command: Command) -> AudioResult<()> {
		self.sender
			.try_borrow_mut()
			.map_err(|_| AudioError::CommandQueueBorrowed)?
			.push(command)
			.map_err(|_| AudioError::CommandQueueFull)
	}
}
