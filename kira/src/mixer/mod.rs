//! Organizes and applies effects to audio.

pub mod effect;
pub(crate) mod effect_slot;
mod track;

pub use track::{
	handle::SendTrackHandle, handle::SubTrackHandle, sends::TrackSends, SendTrackId,
	SendTrackSettings, SubTrackId, SubTrackSettings, TrackIndex,
};
pub(crate) use track::{Track, TrackKind};
