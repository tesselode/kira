use std::{error::Error, fmt::Display};

use cpal::{BuildStreamError, PlayStreamError, SupportedStreamConfigsError};
use lewton::VorbisError;

#[derive(Debug)]
pub enum ConductorError {
	NoDefaultOutputDevice,
	SupportedStreamConfigsError(SupportedStreamConfigsError),
	NoSupportedAudioConfig,
	BuildStreamError(BuildStreamError),
	PlayStreamError(PlayStreamError),
	CommandQueueFull,
	UnsupportedChannelConfiguration,
	UnsupportedAudioFileFormat,
	InvalidSequenceLoopPoint,
	IoError(std::io::Error),
	Mp3Error(minimp3::Error),
	VariableMp3SampleRate,
	UnknownMp3SampleRate,
	OggError(VorbisError),
	FlacError(claxon::Error),
	WavError(hound::Error),
}

impl Display for ConductorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ConductorError::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			ConductorError::SupportedStreamConfigsError(error) => {
				f.write_str(&format!("{}", error))
			}
			ConductorError::NoSupportedAudioConfig => {
				f.write_str("No supported audio configurations")
			}
			ConductorError::BuildStreamError(error) => f.write_str(&format!("{}", error)),
			ConductorError::PlayStreamError(error) => f.write_str(&format!("{}", error)),
			ConductorError::CommandQueueFull => {
				f.write_str("Cannot send a command to the audio thread because the queue is full")
			}
			ConductorError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			ConductorError::UnsupportedAudioFileFormat => {
				f.write_str("Only .mp3, .ogg, .flac, and .wav files are supported")
			}
			ConductorError::InvalidSequenceLoopPoint => {
				f.write_str("The loop point of a sequence cannot be at the very end")
			}
			ConductorError::IoError(error) => f.write_str(&format!("{}", error)),
			ConductorError::Mp3Error(error) => f.write_str(&format!("{}", error)),
			ConductorError::VariableMp3SampleRate => {
				f.write_str("mp3s with variable sample rates are not supported")
			}
			ConductorError::UnknownMp3SampleRate => {
				f.write_str("Could not get the sample rate of the mp3")
			}
			ConductorError::OggError(error) => f.write_str(&format!("{}", error)),
			ConductorError::FlacError(error) => f.write_str(&format!("{}", error)),
			ConductorError::WavError(error) => f.write_str(&format!("{}", error)),
		}
	}
}

impl Error for ConductorError {}

impl From<std::io::Error> for ConductorError {
	fn from(error: std::io::Error) -> Self {
		Self::IoError(error)
	}
}

impl From<minimp3::Error> for ConductorError {
	fn from(error: minimp3::Error) -> Self {
		Self::Mp3Error(error)
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

impl From<hound::Error> for ConductorError {
	fn from(error: hound::Error) -> Self {
		Self::WavError(error)
	}
}

impl From<SupportedStreamConfigsError> for ConductorError {
	fn from(error: SupportedStreamConfigsError) -> Self {
		Self::SupportedStreamConfigsError(error)
	}
}

impl From<BuildStreamError> for ConductorError {
	fn from(error: BuildStreamError) -> Self {
		Self::BuildStreamError(error)
	}
}

impl From<PlayStreamError> for ConductorError {
	fn from(error: PlayStreamError) -> Self {
		Self::PlayStreamError(error)
	}
}

pub type ConductorResult<T> = Result<T, ConductorError>;
