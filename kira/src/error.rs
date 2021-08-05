use std::{
	error::Error,
	fmt::{Display, Formatter},
};

/// Errors that can occur when sending a command to a
/// [`Renderer`](super::manager::Renderer).
#[derive(Debug)]
pub enum CommandError {
	/// Could not add a sound because the command queue is full.
	CommandQueueFull,
	/// Could not add a sound because a thread panicked while using the command queue.
	MutexPoisoned,
}

impl Display for CommandError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			CommandError::CommandQueueFull => {
				"Could not add a sound because the command queue is full."
			}
			CommandError::MutexPoisoned => {
				"Could not add a sound because a thread panicked while using the command queue."
			}
		})
	}
}

impl Error for CommandError {}
