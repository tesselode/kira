use flume::Sender;
use thiserror::Error;

use crate::command::{Command, ParameterCommand};

use super::{ParameterId, Tween};

#[derive(Debug, Error)]
pub enum ParameterHandleError {
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

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

	pub fn set(
		&mut self,
		value: f64,
		tween: impl Into<Option<Tween>>,
	) -> Result<(), ParameterHandleError> {
		self.command_sender
			.send(ParameterCommand::SetParameter(self.id, value, tween.into()).into())
			.map_err(|_| ParameterHandleError::BackendDisconnected)
	}
}
