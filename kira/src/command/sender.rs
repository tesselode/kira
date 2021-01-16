use flume::Sender;

use crate::{command::Command, AudioError, AudioResult};

#[derive(Clone)]
pub(crate) struct CommandSender {
	sender: Sender<Command>,
}

impl CommandSender {
	pub fn new(sender: Sender<Command>) -> Self {
		Self { sender }
	}

	pub fn push(&mut self, command: Command) -> AudioResult<()> {
		self.sender
			.send(command)
			.map_err(|_| AudioError::BackendDisconnected)
	}
}
