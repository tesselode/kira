mod handle;

pub use handle::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Key;

pub(crate) struct SpatialScene {
	shared: Arc<SpatialSceneShared>,
}

impl SpatialScene {
	pub(crate) fn new() -> Self {
		Self {
			shared: Arc::new(SpatialSceneShared::new()),
		}
	}

	pub(crate) fn shared(&self) -> Arc<SpatialSceneShared> {
		self.shared.clone()
	}
}

/// A unique identifier for a spatial scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpatialSceneId(pub(crate) Key);

pub(crate) struct SpatialSceneShared {
	removed: AtomicBool,
}

impl SpatialSceneShared {
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
