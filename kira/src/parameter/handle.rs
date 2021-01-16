use flume::Sender;

use crate::{
	command::{Command, ParameterCommand},
	AudioError, AudioResult,
};

use super::{ParameterId, Tween};

pub struct ParameterHandle {
	id: ParameterId,
	command_sender: Sender<Command>,
}

impl ParameterHandle {
	pub(crate) fn new(id: ParameterId, command_sender: Sender<Command>) -> Self {
		Self { id, command_sender }
	}

	pub fn id(&self) -> ParameterId {
		self.id
	}

	pub fn set(&mut self, value: f64, tween: impl Into<Option<Tween>>) -> AudioResult<()> {
		self.command_sender
			.send(ParameterCommand::SetParameter(self.id, value, tween.into()).into())
			.map_err(|_| AudioError::BackendDisconnected)
	}
}
