use std::{error::Error, fmt::Display};

use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};

/// Something that can go wrong.
#[derive(Debug)]
pub enum AudioError {
	/// A default audio output device could not be found.
	NoDefaultOutputDevice,
	/// An error occurred while getting the list of supported
	/// audio output configurations.
	DefaultStreamConfigError(DefaultStreamConfigError),
	/// No supported output configuration could be found.
	NoSupportedAudioConfig,
	/// An error occurred while setting up the audio thread.
	BuildStreamError(BuildStreamError),
	/// An error occurred while starting the audio thread.
	PlayStreamError(PlayStreamError),
	/// The queue that sends signals from the main thread
	/// to the audio thread is full.
	CommandQueueFull,
	/// The backend cannot receive commands because it no
	/// longer exists.
	BackendDisconnected,
	/// A channel that sends events from the audio thread
	/// to the main thread is already being borrowed.
	EventReceiverBorrowed,
	/// Tried to load audio that isn't mono or stereo.
	UnsupportedChannelConfiguration,
	/// Tried to load audio in an unsupported file format.
	UnsupportedAudioFileFormat,
	/// Tried to add a sequence whose loop point is after
	/// all the other steps.
	InvalidSequenceLoopPoint,
	/// An error occurred when interacting with the filesystem.
	IoError(std::io::Error),
	/// An error occurred when loading an mp3 file.
	#[cfg(feature = "mp3")]
	Mp3Error(minimp3::Error),
	/// An mp3 file has multiple sample rates (not supported).
	#[cfg(feature = "mp3")]
	VariableMp3SampleRate,
	/// The sample rate of an mp3 file could not be determined.
	#[cfg(feature = "mp3")]
	UnknownMp3SampleRate,
	/// An error occurred when loading an ogg file.
	#[cfg(feature = "ogg")]
	OggError(lewton::VorbisError),
	/// An error occurred when loading a flac file.
	#[cfg(feature = "flac")]
	FlacError(claxon::Error),
	/// An error occurred when loading a wav file.
	#[cfg(feature = "wav")]
	WavError(hound::Error),
}

impl Display for AudioError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AudioError::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			AudioError::DefaultStreamConfigError(error) => f.write_str(&format!("{}", error)),
			AudioError::NoSupportedAudioConfig => f.write_str("No supported audio configurations"),
			AudioError::BuildStreamError(error) => f.write_str(&format!("{}", error)),
			AudioError::PlayStreamError(error) => f.write_str(&format!("{}", error)),
			AudioError::CommandQueueFull => {
				f.write_str("Cannot send a command to the audio thread because the queue is full")
			}
			AudioError::BackendDisconnected => {
				f.write_str("The backend cannot receive commands because it no longer exists.")
			}
			AudioError::EventReceiverBorrowed => {
				f.write_str("Cannot pop an event from a receiver because the receiver is currently mutably borrowed")
			}
			AudioError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			AudioError::UnsupportedAudioFileFormat => {
				f.write_str("Only .mp3, .ogg, .flac, and .wav files are supported")
			}
			AudioError::InvalidSequenceLoopPoint => {
				f.write_str("The loop point of a sequence cannot be at the very end")
			}
			AudioError::IoError(error) => f.write_str(&format!("{}", error)),
			#[cfg(feature = "mp3")]
			AudioError::Mp3Error(error) => f.write_str(&format!("{}", error)),
			#[cfg(feature = "mp3")]
			AudioError::VariableMp3SampleRate => {
				f.write_str("mp3s with variable sample rates are not supported")
			}
			#[cfg(feature = "mp3")]
			AudioError::UnknownMp3SampleRate => f.write_str("Could not get the sample rate of the mp3"),
			#[cfg(feature = "ogg")]
			AudioError::OggError(error) => f.write_str(&format!("{}", error)),
			#[cfg(feature = "flac")]
			AudioError::FlacError(error) => f.write_str(&format!("{}", error)),
			#[cfg(feature = "wav")]
			AudioError::WavError(error) => f.write_str(&format!("{}", error)),
		}
	}
}

impl Error for AudioError {}

impl From<std::io::Error> for AudioError {
	fn from(error: std::io::Error) -> Self {
		Self::IoError(error)
	}
}

#[cfg(feature = "mp3")]
impl From<minimp3::Error> for AudioError {
	fn from(error: minimp3::Error) -> Self {
		Self::Mp3Error(error)
	}
}

#[cfg(feature = "ogg")]
impl From<lewton::VorbisError> for AudioError {
	fn from(error: lewton::VorbisError) -> Self {
		Self::OggError(error)
	}
}

#[cfg(feature = "flac")]
impl From<claxon::Error> for AudioError {
	fn from(error: claxon::Error) -> Self {
		Self::FlacError(error)
	}
}

#[cfg(feature = "wav")]
impl From<hound::Error> for AudioError {
	fn from(error: hound::Error) -> Self {
		Self::WavError(error)
	}
}

impl From<DefaultStreamConfigError> for AudioError {
	fn from(error: DefaultStreamConfigError) -> Self {
		Self::DefaultStreamConfigError(error)
	}
}

impl From<BuildStreamError> for AudioError {
	fn from(error: BuildStreamError) -> Self {
		Self::BuildStreamError(error)
	}
}

impl From<PlayStreamError> for AudioError {
	fn from(error: PlayStreamError) -> Self {
		Self::PlayStreamError(error)
	}
}

/// A wrapper around the standard [`Result`](Result)
/// type that always has an [`AudioError`](AudioError)
/// as its error type.
pub type AudioResult<T> = Result<T, AudioError>;
