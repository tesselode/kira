//! Things that can go wrong with sequences.

use thiserror::Error;

/// Invalid states a sequence can be in.
#[derive(Debug, Error)]
pub enum SequenceError {
	/// The sequence has a looping section without a `wait`
	/// or `wait_for_interval` command.
	///
	/// This is invalid because if this sequence were to run,
	/// the looping section would lock up the audio thread
	/// by processing forever.
	#[error("The looping section of a sequence must have a wait-related command")]
	InfiniteLoop,
}
