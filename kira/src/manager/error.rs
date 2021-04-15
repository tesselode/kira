use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

use crate::sound::data::static_sound::error::StaticSoundDataFromFileError;

/// Things that can go wrong when creating an `AudioManager`.
#[derive(Debug, Error)]
pub enum SetupError {
	/// A default audio output device could not be determined.
	#[error("Cannot find the default audio output device")]
	NoDefaultOutputDevice,

	/// An error occurred when getting the default output configuration.
	#[error("{0}")]
	DefaultStreamConfigError(#[from] DefaultStreamConfigError),

	/// An error occured when building the audio stream.
	#[error("{0}")]
	BuildStreamError(#[from] BuildStreamError),

	/// An error occured when starting the audio stream.
	#[error("{0}")]
	PlayStreamError(#[from] PlayStreamError),
}

/// Things that can go wrong when loading a sound from a file.
#[derive(Debug, Error)]
pub enum LoadSoundError {
	/// An error occurred while loading the sound data.
	#[error("{0}")]
	StaticSoundDataFromFileError(#[from] StaticSoundDataFromFileError),

	/// The command queue is full, so commands cannot be sent to the audio thread.
	#[error("The command queue is full, so commands cannot be sent to the audio thread")]
	CommandQueueFullError,
}
