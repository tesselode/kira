use thiserror::Error;

#[derive(Debug, Error)]
pub enum SequenceError {
	#[error("The looping section of a sequence must have a wait-related command")]
	InfiniteLoop,
}
