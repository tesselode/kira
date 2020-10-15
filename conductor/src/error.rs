use std::{error::Error, fmt::Display};

use lewton::VorbisError;

#[derive(Debug)]
pub enum ConductorError {
	CommandQueueFull,
	UnsupportedChannelConfiguration,
	UnsupportedAudioFileFormat,
	IoError(std::io::Error),
	OggError(VorbisError),
	FlacError(claxon::Error),
}

impl Display for ConductorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ConductorError::CommandQueueFull => {
				f.write_str("Cannot send a command to the audio thread because the queue is full")
			}
			ConductorError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			ConductorError::UnsupportedAudioFileFormat => {
				f.write_str("Only .ogg and .flac files are supported")
			}
			ConductorError::IoError(error) => f.write_str(&format!("{}", error)),
			ConductorError::OggError(error) => f.write_str(&format!("{}", error)),
			ConductorError::FlacError(error) => f.write_str(&format!("{}", error)),
		}
	}
}

impl Error for ConductorError {}

impl From<std::io::Error> for ConductorError {
	fn from(error: std::io::Error) -> Self {
		Self::IoError(error)
	}
}

impl From<VorbisError> for ConductorError {
	fn from(error: VorbisError) -> Self {
		Self::OggError(error)
	}
}

impl From<claxon::Error> for ConductorError {
	fn from(error: claxon::Error) -> Self {
		Self::FlacError(error)
	}
}

pub type ConductorResult<T> = Result<T, ConductorError>;
