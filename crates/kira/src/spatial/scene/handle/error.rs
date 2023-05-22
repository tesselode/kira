use std::{
	error::Error,
	fmt::{Display, Formatter},
};

use crate::CommandError;

/// Errors that can occur when creating a emitter.
#[derive(Debug)]
#[non_exhaustive]
pub enum AddEmitterError {
	/// Could not add a emitter because the maximum number of emitters has been reached.
	EmitterLimitReached,
	/// An error occured when sending a command to the audio thread.
	CommandError(CommandError),
}

impl Display for AddEmitterError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AddEmitterError::EmitterLimitReached => f.write_str(
				"Could not add a emitter because the maximum number of emitters has been reached.",
			),
			AddEmitterError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for AddEmitterError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			AddEmitterError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for AddEmitterError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}

/// Errors that can occur when creating a listener.
#[derive(Debug)]
#[non_exhaustive]
pub enum AddListenerError {
	/// Could not add a listener because the maximum number of listeners has been reached.
	ListenerLimitReached,
	/// An error occured when sending a command to the audio thread.
	CommandError(CommandError),
}

impl Display for AddListenerError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AddListenerError::ListenerLimitReached => f.write_str(
				"Could not add a listener because the maximum number of listeners has been reached.",
			),
			AddListenerError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for AddListenerError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			AddListenerError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for AddListenerError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}
