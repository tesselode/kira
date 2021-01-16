use thiserror::Error;

use crate::{
	arrangement::ArrangementId,
	group::GroupId,
	metronome::MetronomeId,
	mixer::SubTrackId,
	parameter::ParameterId,
	sound::{error::SoundFromFileError, SoundId},
};

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
