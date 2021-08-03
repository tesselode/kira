//! Errors that can occur when using an [`AudioManager`](super::AudioManager).

use thiserror::Error;

use crate::error::CommandError;

/// Errors that can occur when adding a sound to the renderer.
#[derive(Debug, Error)]
pub enum AddSoundError {
	/// Could not add a sound because the maximum number of sounds has been reached.
	#[error("Could not add a sound because the maximum number of sounds has been reached.")]
	SoundLimitReached,
	/// An error occured when sending a command to the renderer.
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

/// Errors that can occur when loading a sound from a file
/// and sending it to the renderer.
#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
#[derive(Debug, Error)]
pub enum LoadSoundError {
	/// An error occurred when loading a sound from a file.
	#[error("{0}")]
	FromFileError(#[from] crate::sound::data::static_sound::FromFileError),
	/// An error occurred when sending the sound to the renderer.
	#[error("{0}")]
	AddSoundError(#[from] AddSoundError),
}

/// Errors that can occur when creating a parameter.
#[derive(Debug, Error)]
pub enum AddParameterError {
	/// Could not add a parameter because the maximum number of parameters has been reached.
	#[error(
		"Could not add a parameter because the maximum number of parameters has been reached."
	)]
	ParameterLimitReached,
	/// An error occured when sending a command to the renderer.
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

/// Errors that can occur when creating a mixer sub-track.
#[derive(Debug, Error)]
pub enum AddSubTrackError {
	/// Could not add a sub-track because the maximum number of sub-tracks has been reached.
	#[error(
		"Could not add a sub-track because the maximum number of sub-tracks has been reached."
	)]
	SubTrackLimitReached,
	/// An error occured when sending a command to the renderer.
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

/// Errors that can occur when creating a clock.
#[derive(Debug, Error)]
pub enum AddClockError {
	/// Could not add a clock because the maximum number of clocks has been reached.
	#[error("Could not add a clock because the maximum number of clocks has been reached.")]
	ClockLimitReached,
	/// An error occured when sending a command to the renderer.
	#[error("{0}")]
	CommandError(#[from] CommandError),
}

/// Errors that can occur when creating a audio stream.
#[derive(Debug, Error)]
pub enum AddAudioStreamError {
	/// Could not add a audio stream because the maximum number of audio streams has been reached.
	#[error("Could not add a audio stream because the maximum number of audio streams has been reached.")]
	AudioStreamLimitReached,
	/// An error occured when sending a command to the renderer.
	#[error("{0}")]
	CommandError(#[from] CommandError),
}
