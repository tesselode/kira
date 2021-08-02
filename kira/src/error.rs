use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
	#[error("Could not add a sound because the command queue is full.")]
	CommandQueueFull,
	#[error("Could not add a sound because a thread panicked while using the command queue.")]
	MutexPoisoned,
}
