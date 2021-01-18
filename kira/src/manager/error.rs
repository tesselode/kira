//! Things that can go wrong when using an [`AudioManager`](super::AudioManager).

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

/// Things that can go wrong when creating an `AudioManager`.
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

/// Things that can go wrong when adding a sound to the audio thread.
#[derive(Debug, Error)]
pub enum AddSoundError {
	/// The maximum sound limit has been reached.
	#[error("Cannot add a sound because the max number of sounds has been reached")]
	SoundLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when loading a sound from a file
/// and sending it to the audio thread.
#[derive(Debug, Error)]
pub enum LoadSoundError {
	/// An error occurred when sending the sound to the audio thread.
	#[error("{0}")]
	AddSoundError(#[from] AddSoundError),

	/// An error occurred when loading a sound from a file.
	#[error("{0}")]
	SoundFromFileError(#[from] SoundFromFileError),
}

/// Things that can go wrong when removing a sound from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveSoundError {
	/// No sound with the specified ID exists.
	#[error("The sound with the specified ID does not exist")]
	NoSoundWithId(SoundId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when adding an arrangement to the audio thread.
#[derive(Debug, Error)]
pub enum AddArrangementError {
	/// The maximum arrangement limit has been reached.
	#[error("Cannot add an arrangement because the max number of arrangements has been reached")]
	ArrangementLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when removing an arrangement from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveArrangementError {
	/// No arrangement with the specified ID exists.
	#[error("The arrangement with the specified ID does not exist")]
	NoArrangementWithId(ArrangementId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when adding a metronome to the audio thread.
#[derive(Debug, Error)]
pub enum AddMetronomeError {
	/// The maximum metronome limit has been reached.
	#[error("Cannot add a metronome because the max number of metronomes has been reached")]
	MetronomeLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when removing a metronome from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveMetronomeError {
	/// No metronome with the specified ID exists.
	#[error("The metronome with the specified ID does not exist")]
	NoMetronomeWithId(MetronomeId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when adding a group to the audio thread.
#[derive(Debug, Error)]
pub enum AddGroupError {
	/// The maximum group limit has been reached.
	#[error("Cannot add an group because the max number of groups has been reached")]
	GroupLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when removing a group from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveGroupError {
	/// No group with the specified ID exists.
	#[error("The group with the specified ID does not exist")]
	NoGroupWithId(GroupId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when adding a parameter to the audio thread.
#[derive(Debug, Error)]
pub enum AddParameterError {
	/// The maximum parameter limit has been reached.
	#[error("Cannot add an parameter because the max number of parameters has been reached")]
	ParameterLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when removing a parameter from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveParameterError {
	/// No parameter with the specified ID exists.
	#[error("The parameter with the specified ID does not exist")]
	NoParameterWithId(ParameterId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when adding a mixer track to the audio thread.
#[derive(Debug, Error)]
pub enum AddTrackError {
	/// The maximum track limit has been reached.
	#[error("Cannot add an track because the max number of tracks has been reached")]
	TrackLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when removing a mixer track from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveTrackError {
	/// No mixer track with the specified ID exists.
	#[error("The track with the specified ID does not exist")]
	NoTrackWithId(SubTrackId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when adding an audio stream to the audio thread.
#[derive(Debug, Error)]
pub enum AddStreamError {
	/// The maximum audio stream limit has been reached.
	#[error("Cannot add a stream because the max number of streams has been reached")]
	StreamLimitReached,

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when removing an audio stream from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveStreamError {
	/// No audio stream with the specified ID exists.
	#[error("The stream with the specified ID does not exist")]
	NoStreamWithId(AudioStreamId),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Things that can go wrong when starting a sequence.
#[derive(Debug, Error)]
pub enum StartSequenceError {
	/// The sequence is in an invalid state.
	#[error("{0}")]
	SequenceError(#[from] SequenceError),

	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}
