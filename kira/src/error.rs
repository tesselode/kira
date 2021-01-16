use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

use crate::{
	arrangement::ArrangementId, group::GroupId, metronome::MetronomeId, mixer::SubTrackId,
	parameter::ParameterId, sound::SoundId,
};

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

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,

	#[error(
		"Cannot pop an event from a receiver because the receiver is currently mutably borrowed"
	)]
	EventReceiverBorrowed,

	#[error("Cannot add a sound because the max number of sounds has been reached")]
	SoundLimitReached,

	#[error("Cannot add an arrangement because the max number of arrangements has been reached")]
	ArrangementLimitReached,

	#[error("Cannot add an parameter because the max number of parameters has been reached")]
	ParameterLimitReached,

	#[error("Cannot add an track because the max number of tracks has been reached")]
	TrackLimitReached,

	#[error("Cannot add an group because the max number of groups has been reached")]
	GroupLimitReached,

	#[error("Cannot add an metronome because the max number of metronomes has been reached")]
	MetronomeLimitReached,

	#[error("The sound with the specified ID does not exist")]
	NoSoundWithId(SoundId),

	#[error("The arrangement with the specified ID does not exist")]
	NoArrangementWithId(ArrangementId),

	#[error("The parameter with the specified ID does not exist")]
	NoParameterWithId(ParameterId),

	#[error("The track with the specified ID does not exist")]
	NoTrackWithId(SubTrackId),

	#[error("The group with the specified ID does not exist")]
	NoGroupWithId(GroupId),

	#[error("The metronome with the specified ID does not exist")]
	NoMetronomeWithId(MetronomeId),

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
