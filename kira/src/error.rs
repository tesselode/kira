use thiserror::Error;

/// Errors that can occur when sending a command to the
/// audio thread.
#[derive(Debug, Error)]
pub enum CommandError {
	/// Could not add a sound because the command queue is full.
	#[error("Could not add a sound because the command queue is full.")]
	CommandQueueFull,
	/// Could not add a sound because a thread panicked while using the command queue.
	#[error("Could not add a sound because a thread panicked while using the command queue.")]
	MutexPoisoned,
}
