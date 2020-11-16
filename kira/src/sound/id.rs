use std::{
	hash::Hash,
	sync::atomic::{AtomicUsize, Ordering},
};

use crate::mixer::TrackIndex;

use super::SoundMetadata;

static NEXT_SOUND_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a `Sound`.
///
/// You cannot create this manually - a `SoundId` is returned
/// when you load a sound with an `AudioManager`.
#[derive(Debug, Copy, Clone)]
pub struct SoundId {
	index: usize,
	duration: f64,
	default_track_index: TrackIndex,
	metadata: SoundMetadata,
}

impl SoundId {
	pub fn duration(&self) -> f64 {
		self.duration
	}

	pub fn default_track_index(&self) -> TrackIndex {
		self.default_track_index
	}

	pub fn metadata(&self) -> &SoundMetadata {
		&self.metadata
	}
}

impl PartialEq for SoundId {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index
	}
}

impl Eq for SoundId {}

impl Hash for SoundId {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.index.hash(state);
	}
}

impl SoundId {
	pub(crate) fn new(
		duration: f64,
		default_track_index: TrackIndex,
		metadata: SoundMetadata,
	) -> Self {
		let index = NEXT_SOUND_INDEX.fetch_add(1, Ordering::Relaxed);
		Self {
			index,
			duration,
			default_track_index,
			metadata,
		}
	}
}
