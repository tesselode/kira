//! Errors that can occur when using an [`AudioManager`](super::AudioManager).

use std::{
	error::Error,
	fmt::{Display, Formatter},
};

/// Errors that can occur when playing a sound.
#[derive(Debug)]
pub enum PlaySoundError<E> {
	/// Could not play a sound because the maximum number of sounds has been reached.
	SoundLimitReached,
	/// An error occurred when initializing the sound.
	IntoSoundError(E),
}

impl<E> Display for PlaySoundError<E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			PlaySoundError::SoundLimitReached => f.write_str(
				"Could not play a sound because the maximum number of sounds has been reached.",
			),
			PlaySoundError::IntoSoundError(_) => {
				f.write_str("An error occurred when initializing the sound.")
			}
		}
	}
}

impl<E: std::fmt::Debug> Error for PlaySoundError<E> {}
