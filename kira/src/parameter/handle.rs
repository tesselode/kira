//! An interface for controlling parameters.

use flume::Sender;
use thiserror::Error;

use crate::command::{Command, ParameterCommand};

use super::{tween::Tween, ParameterId};

/// Something that can go wrong when using a [`ParameterHandle`]
/// to control a parameter.
#[derive(Debug, Error)]
pub enum ParameterHandleError {
	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Allows you to control a parameter.
pub struct ParameterHandle {
	id: ParameterId,
	command_sender: Sender<Command>,
}

impl ParameterHandle {
	pub(crate) fn new(id: ParameterId, command_sender: Sender<Command>) -> Self {
		Self { id, command_sender }
	}

	/// Returns the ID of the parameter.
	pub fn id(&self) -> ParameterId {
		self.id
	}

	/// Sets the parameter to a value with an optional tween.
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
