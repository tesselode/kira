pub mod data;
pub mod handle;
pub mod instance;
pub mod metadata;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Index;

use self::data::SoundData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(pub(crate) Index);

pub(crate) struct SoundShared {
	removed: AtomicBool,
}

impl SoundShared {
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

pub(crate) struct Sound {
	pub data: Arc<dyn SoundData>,
	pub shared: Arc<SoundShared>,
}
