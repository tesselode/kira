//! Errors that can occur when using an [`AudioManager`](super::AudioManager).

use std::{
	error::Error,
	fmt::{Display, Formatter},
};

use crate::error::CommandError;

/// Errors that can occur when playing a sound.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlaySoundError<E> {
	/// Could not play a sound because the maximum number of sounds has been reached.
	SoundLimitReached,
	/// An error occurred when initializing the sound.
	IntoSoundError(E),
	/// An error occured when sending a command to the audio thread.
	CommandError(CommandError),
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
			PlaySoundError::CommandError(error) => error.fmt(f),
		}
	}
}

impl<E: std::fmt::Debug> Error for PlaySoundError<E> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			PlaySoundError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl<E> From<CommandError> for PlaySoundError<E> {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}

/// Errors that can occur when creating a mixer sub-track.
#[derive(Debug)]
#[non_exhaustive]
pub enum AddSubTrackError {
	/// Could not add a sub-track because the maximum number of sub-tracks has been reached.
	SubTrackLimitReached,
	/// An error occured when sending a command to the audio thread.
	CommandError(CommandError),
}

impl Display for AddSubTrackError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AddSubTrackError::SubTrackLimitReached => f.write_str("Could not add a sub-track because the maximum number of sub-tracks has been reached."),
			AddSubTrackError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for AddSubTrackError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			AddSubTrackError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for AddSubTrackError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}

/// Errors that can occur when creating a clock.
#[derive(Debug)]
#[non_exhaustive]
pub enum AddClockError {
	/// Could not add a clock because the maximum number of clocks has been reached.
	ClockLimitReached,
	/// An error occured when sending a command to the audio thread.
	CommandError(CommandError),
}

impl Display for AddClockError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AddClockError::ClockLimitReached => f.write_str(
				"Could not add a clock because the maximum number of clocks has been reached.",
			),
			AddClockError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for AddClockError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			AddClockError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for AddClockError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}

/// Errors that can occur when creating a spatial scene.
#[derive(Debug)]
#[non_exhaustive]
pub enum AddSpatialSceneError {
	/// Could not add a spatial scene because the maximum number of spatial scenes has been reached.
	SpatialSceneLimitReached,
	/// An error occured when sending a command to the audio thread.
	CommandError(CommandError),
}

impl Display for AddSpatialSceneError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AddSpatialSceneError::SpatialSceneLimitReached => f.write_str(
				"Could not add a spatial scene because the maximum number of spatial scenes has been reached.",
			),
			AddSpatialSceneError::CommandError(error) => error.fmt(f),
		}
	}
}

impl Error for AddSpatialSceneError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			AddSpatialSceneError::CommandError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<CommandError> for AddSpatialSceneError {
	fn from(v: CommandError) -> Self {
		Self::CommandError(v)
	}
}
