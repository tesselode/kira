use super::id::SubTrackId;

/// Represents a mixer track.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TrackIndex {
	/// The main track.
	///
	/// All sub-tracks are sent to the main track as input,
	/// and the output of the main track is what you hear.
	Main,
	/// A sub-track.
	///
	/// Sub-tracks are useful for adjusting the volumes of
	/// and applying effects to certain kinds of sounds.
	/// For example, in a game, you may have one sub-track
	/// for sound effects and another for music.
	Sub(SubTrackId),
}
