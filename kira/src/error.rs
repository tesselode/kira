use std::{error::Error, fmt::Display};

use cpal::{BuildStreamError, PlayStreamError, SupportedStreamConfigsError};
use lewton::VorbisError;

#[derive(Debug)]
pub enum KiraError {
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

impl Display for KiraError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			KiraError::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			KiraError::SupportedStreamConfigsError(error) => {
				f.write_str(&format!("{}", error))
			}
			KiraError::NoSupportedAudioConfig => {
				f.write_str("No supported audio configurations")
			}
			KiraError::BuildStreamError(error) => f.write_str(&format!("{}", error)),
			KiraError::PlayStreamError(error) => f.write_str(&format!("{}", error)),
			KiraError::CommandQueueFull => {
				f.write_str("Cannot send a command to the audio thread because the queue is full")
			}
			KiraError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			KiraError::UnsupportedAudioFileFormat => {
				f.write_str("Only .mp3, .ogg, .flac, and .wav files are supported")
			}
			KiraError::InvalidSequenceLoopPoint => {
				f.write_str("The loop point of a sequence cannot be at the very end")
			}
			KiraError::IoError(error) => f.write_str(&format!("{}", error)),
			KiraError::Mp3Error(error) => f.write_str(&format!("{}", error)),
			KiraError::VariableMp3SampleRate => {
				f.write_str("mp3s with variable sample rates are not supported")
			}
			KiraError::UnknownMp3SampleRate => {
				f.write_str("Could not get the sample rate of the mp3")
			}
			KiraError::OggError(error) => f.write_str(&format!("{}", error)),
			KiraError::FlacError(error) => f.write_str(&format!("{}", error)),
			KiraError::WavError(error) => f.write_str(&format!("{}", error)),
		}
	}
}

impl Error for KiraError {}

impl From<std::io::Error> for KiraError {
	fn from(error: std::io::Error) -> Self {
		Self::IoError(error)
	}
}

impl From<minimp3::Error> for KiraError {
	fn from(error: minimp3::Error) -> Self {
		Self::Mp3Error(error)
	}
}

impl From<VorbisError> for KiraError {
	fn from(error: VorbisError) -> Self {
		Self::OggError(error)
	}
}

impl From<claxon::Error> for KiraError {
	fn from(error: claxon::Error) -> Self {
		Self::FlacError(error)
	}
}

impl From<hound::Error> for KiraError {
	fn from(error: hound::Error) -> Self {
		Self::WavError(error)
	}
}

impl From<SupportedStreamConfigsError> for KiraError {
	fn from(error: SupportedStreamConfigsError) -> Self {
		Self::SupportedStreamConfigsError(error)
	}
}

impl From<BuildStreamError> for KiraError {
	fn from(error: BuildStreamError) -> Self {
		Self::BuildStreamError(error)
	}
}

impl From<PlayStreamError> for KiraError {
	fn from(error: PlayStreamError) -> Self {
		Self::PlayStreamError(error)
	}
}

pub type KiraResult<T> = Result<T, KiraError>;
