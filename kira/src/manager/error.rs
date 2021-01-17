use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

use crate::{
	arrangement::ArrangementId,
	audio_stream::AudioStreamId,
	group::GroupId,
	metronome::MetronomeId,
	mixer::SubTrackId,
	parameter::ParameterId,
	sequence::error::SequenceError,
	sound::{error::SoundFromFileError, SoundId},
};

#[derive(Debug, Error)]
pub enum SetupError {
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
}

#[derive(Debug, Error)]
pub enum AddSoundError {
	#[error("Cannot add a sound because the max number of sounds has been reached")]
	SoundLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum LoadSoundError {
	#[error("{0}")]
	AddSoundError(#[from] AddSoundError),

	#[error("{0}")]
	SoundFromFileError(#[from] SoundFromFileError),
}

#[derive(Debug, Error)]
pub enum RemoveSoundError {
	#[error("The sound with the specified ID does not exist")]
	NoSoundWithId(SoundId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum AddArrangementError {
	#[error("Cannot add an arrangement because the max number of arrangements has been reached")]
	ArrangementLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum RemoveArrangementError {
	#[error("The arrangement with the specified ID does not exist")]
	NoArrangementWithId(ArrangementId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum AddMetronomeError {
	#[error("Cannot add a metronome because the max number of metronomes has been reached")]
	MetronomeLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum RemoveMetronomeError {
	#[error("The metronome with the specified ID does not exist")]
	NoMetronomeWithId(MetronomeId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum AddGroupError {
	#[error("Cannot add an group because the max number of groups has been reached")]
	GroupLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum RemoveGroupError {
	#[error("The group with the specified ID does not exist")]
	NoGroupWithId(GroupId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum AddParameterError {
	#[error("Cannot add an parameter because the max number of parameters has been reached")]
	ParameterLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum RemoveParameterError {
	#[error("The parameter with the specified ID does not exist")]
	NoParameterWithId(ParameterId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum AddTrackError {
	#[error("Cannot add an track because the max number of tracks has been reached")]
	TrackLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum RemoveTrackError {
	#[error("The track with the specified ID does not exist")]
	NoTrackWithId(SubTrackId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum AddStreamError {
	#[error("Cannot add a stream because the max number of streams has been reached")]
	StreamLimitReached,

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum RemoveStreamError {
	#[error("The stream with the specified ID does not exist")]
	NoStreamWithId(AudioStreamId),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Error)]
pub enum StartSequenceError {
	#[error("{0}")]
	SequenceError(#[from] SequenceError),

	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}
