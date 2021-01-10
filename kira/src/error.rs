use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

/// Something that can go wrong.
#[derive(Debug, Error)]
pub enum AudioError {
	#[error("Cannot find the default audio output device")]
	NoDefaultOutputDevice,

	#[error("{0}")]
	DefaultStreamConfigError(#[from] DefaultStreamConfigError),

	#[error("No supported audio configurations")]
	NoSupportedAudioConfig,

	#[error("{0}")]
	BuildStreamError(#[from] BuildStreamError),

	#[error("{0}")]
	PlayStreamError(#[from] PlayStreamError),

	#[error("Cannot send a command to the audio thread because the queue is full")]
	CommandQueueFull,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,

	#[error(
		"Cannot pop an event from a receiver because the receiver is currently mutably borrowed"
	)]
	EventReceiverBorrowed,

	#[error("Only mono and stereo audio is supported")]
	UnsupportedChannelConfiguration,

	#[error("Only .mp3, .ogg, .flac, and .wav files are supported")]
	UnsupportedAudioFileFormat,

	#[error("The loop point of a sequence cannot be at the very end")]
	InvalidSequenceLoopPoint,

	#[error("{0}")]
	IoError(#[from] std::io::Error),

	#[cfg(feature = "mp3")]
	#[error("{0}")]
	Mp3Error(#[from] minimp3::Error),

	#[cfg(feature = "mp3")]
	#[error("mp3s with variable sample rates are not supported")]
	VariableMp3SampleRate,

	#[cfg(feature = "mp3")]
	#[error("Could not get the sample rate of the mp3")]
	UnknownMp3SampleRate,

	#[cfg(feature = "ogg")]
	#[error("{0}")]
	OggError(#[from] lewton::VorbisError),

	#[cfg(feature = "flac")]
	#[error("{0}")]
	FlacError(#[from] claxon::Error),

	#[cfg(feature = "wav")]
	#[error("{0}")]
	WavError(#[from] hound::Error),
}

/// A wrapper around the standard [`Result`](Result)
/// type that always has an [`AudioError`](AudioError)
/// as its error type.
pub type AudioResult<T> = Result<T, AudioError>;
