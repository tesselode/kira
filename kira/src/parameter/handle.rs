use crate::{
	command::{sender::CommandSender, ParameterCommand},
	AudioResult,
};

use super::{ParameterId, Tween};

pub struct ParameterHandle {
	id: ParameterId,
	command_sender: CommandSender,
}

impl ParameterHandle {
	pub(crate) fn new(id: ParameterId, command_sender: CommandSender) -> Self {
		Self { id, command_sender }
	}

	pub fn id(&self) -> ParameterId {
		self.id
	}

	pub fn remove(&mut self) -> AudioResult<()> {
		self.command_sender
			.push(ParameterCommand::RemoveParameter(self.id).into())
	}

	pub fn set(&mut self, value: f64, tween: impl Into<Option<Tween>>) -> AudioResult<()> {
		self.command_sender
			.push(ParameterCommand::SetParameter(self.id, value, tween.into()).into())
	}
}
