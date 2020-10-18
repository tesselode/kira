use std::{
	hash::Hash,
	sync::atomic::{AtomicUsize, Ordering},
};

use super::SoundMetadata;

static NEXT_SOUND_INDEX: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a `Sound`.
///
/// You cannot create this manually - a `SoundId` is returned
/// when you load a sound with a `Project`.
#[derive(Debug, Copy, Clone)]
pub struct SoundId {
	index: usize,
	duration: f64,
	metadata: SoundMetadata,
}

impl SoundId {
	pub fn duration(&self) -> f64 {
		self.duration
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
	pub(crate) fn new(duration: f64, metadata: SoundMetadata) -> Self {
		let index = NEXT_SOUND_INDEX.fetch_add(1, Ordering::Relaxed);
		Self {
			index,
			duration,
			metadata,
		}
	}
}
