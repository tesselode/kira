use thiserror::Error;

#[derive(Debug, Error)]
pub enum SequenceError {
	#[error("The loop point of a sequence cannot be at the very end")]
	InvalidLoopPoint,
}
