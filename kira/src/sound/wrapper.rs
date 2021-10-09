use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use super::Sound;

pub(crate) struct SoundWrapperShared {
	removed: AtomicBool,
}

impl SoundWrapperShared {
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

pub(crate) struct SoundWrapper {
	pub sound: Box<dyn Sound>,
	pub shared: Arc<SoundWrapperShared>,
}
