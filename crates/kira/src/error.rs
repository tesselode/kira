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

/// An error that is returned when a resource cannot be added because the
/// maximum capacity for that resource has been reached.
///
/// You can adjust these capacities using [`Capacities`](crate::manager::Capacities).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceLimitReached;

impl Display for ResourceLimitReached {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Could not add a resource because the maximum capacity for that resource has been reached")
	}
}

impl Error for ResourceLimitReached {}
