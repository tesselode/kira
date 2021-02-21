//! Things that can go wrong when using an [`AudioManager`](super::AudioManager).

use cpal::{BuildStreamError, DefaultStreamConfigError, PlayStreamError};
use thiserror::Error;

use crate::{
	arrangement::ArrangementId,
	audio_stream::AudioStreamId,
	command::producer::CommandError,
	group::GroupId,
	metronome::MetronomeId,
	mixer::{SendTrackId, SubTrackId, TrackIndex},
	parameter::ParameterId,
	sequence::error::SequenceError,
	sound::{error::SoundFromFileError, SoundId},
};

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

/// Things that can go wrong when adding a sound to the audio thread.
#[derive(Debug, Error)]
pub enum AddSoundError {
	/// The maximum sound limit has been reached.
	#[error("Cannot add a sound because the max number of sounds has been reached")]
	SoundLimitReached,

	/// The default track for the sound does not exist.
	#[error("The default track for the sound does not exist")]
	NoTrackWithIndex(TrackIndex),

	/// The sound belongs to a group that does not exist.
	#[error("The sound belongs to a group that does not exist")]
	NoGroupWithId(GroupId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
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

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when adding an arrangement to the audio thread.
#[derive(Debug, Error)]
pub enum AddArrangementError {
	/// The maximum arrangement limit has been reached.
	#[error("Cannot add an arrangement because the max number of arrangements has been reached")]
	ArrangementLimitReached,

	/// The default track for the arrangement does not exist.
	#[error("The default track for the arrangement does not exist")]
	NoTrackWithIndex(TrackIndex),

	/// The arrangement belongs to a group that does not exist.
	#[error("The arrangement belongs to a group that does not exist")]
	NoGroupWithId(GroupId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when removing an arrangement from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveArrangementError {
	/// No arrangement with the specified ID exists.
	#[error("The arrangement with the specified ID does not exist")]
	NoArrangementWithId(ArrangementId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when adding a metronome to the audio thread.
#[derive(Debug, Error)]
pub enum AddMetronomeError {
	/// The maximum metronome limit has been reached.
	#[error("Cannot add a metronome because the max number of metronomes has been reached")]
	MetronomeLimitReached,

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when removing a metronome from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveMetronomeError {
	/// No metronome with the specified ID exists.
	#[error("The metronome with the specified ID does not exist")]
	NoMetronomeWithId(MetronomeId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when adding a group to the audio thread.
#[derive(Debug, Error)]
pub enum AddGroupError {
	/// The maximum group limit has been reached.
	#[error("Cannot add an group because the max number of groups has been reached")]
	GroupLimitReached,

	/// The group belongs to a parent group that does not exist.
	#[error("The group belongs to a parent group that does not exist")]
	NoGroupWithId(GroupId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when removing a group from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveGroupError {
	/// No group with the specified ID exists.
	#[error("The group with the specified ID does not exist")]
	NoGroupWithId(GroupId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when adding a parameter to the audio thread.
#[derive(Debug, Error)]
pub enum AddParameterError {
	/// The maximum parameter limit has been reached.
	#[error("Cannot add an parameter because the max number of parameters has been reached")]
	ParameterLimitReached,

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when removing a parameter from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveParameterError {
	/// No parameter with the specified ID exists.
	#[error("The parameter with the specified ID does not exist")]
	NoParameterWithId(ParameterId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when adding a mixer track to the audio thread.
#[derive(Debug, Error)]
pub enum AddTrackError {
	/// The maximum track limit has been reached.
	#[error("Cannot add an track because the max number of tracks has been reached")]
	TrackLimitReached,

	/// The track's parent track does not exist.
	#[error("The track's parent track does not exist")]
	NoTrackWithIndex(TrackIndex),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when removing a mixer track from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveTrackError {
	/// No mixer sub-track with the specified ID exists.
	#[error("The sub-track with the specified ID does not exist")]
	NoSubTrackWithId(SubTrackId),

	/// No mixer send track with the specified ID exists.
	#[error("The send track with the specified ID does not exist")]
	NoSendTrackWithId(SendTrackId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when adding an audio stream to the audio thread.
#[derive(Debug, Error)]
pub enum AddStreamError {
	/// The maximum audio stream limit has been reached.
	#[error("Cannot add a stream because the max number of streams has been reached")]
	StreamLimitReached,

	/// The specified track for the stream does not exist.
	#[error("The specified track for the stream does not exist")]
	NoTrackWithIndex(TrackIndex),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when removing an audio stream from the
/// audio thread.
#[derive(Debug, Error)]
pub enum RemoveStreamError {
	/// No audio stream with the specified ID exists.
	#[error("The stream with the specified ID does not exist")]
	NoStreamWithId(AudioStreamId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Things that can go wrong when starting a sequence.
#[derive(Debug, Error)]
pub enum StartSequenceError {
	/// The sequence is in an invalid state.
	#[error("{0}")]
	SequenceError(#[from] SequenceError),

	/// The sequence belongs to a group that does not exist.
	#[error("The sequence belongs to a group that does not exist")]
	NoGroupWithId(GroupId),

	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}
