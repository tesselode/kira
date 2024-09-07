mod main;
mod send;
mod sub;

pub use main::*;
pub use send::*;
pub use sub::*;

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub(crate) struct TrackShared {
	removed: AtomicBool,
}

impl TrackShared {
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}
