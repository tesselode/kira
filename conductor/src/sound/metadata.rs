use crate::{duration::Duration, tempo::Tempo};

/// Useful info about a `Sound`.
///
/// This is set entirely by the user when loading a sound
/// and can be accessed via `SoundId`s.
#[derive(Debug, Default, Copy, Clone)]
pub struct SoundMetadata {
	pub tempo: Option<Tempo>,
	pub semantic_duration: Option<Duration>,
}
