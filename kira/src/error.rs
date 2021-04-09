use thiserror::Error;

#[derive(Debug, Error)]
#[error("The command queue is full, so commands cannot be sent to the audio thread")]
pub struct CommandQueueFullError;
