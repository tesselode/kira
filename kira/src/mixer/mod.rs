//! Provides an interface for organizing and applying effects to sounds.

pub mod effect;
pub(crate) mod effect_slot;
mod track;

pub(crate) use track::Track;
pub use track::{SubTrackId, TrackHandle, TrackIndex, TrackSettings};
