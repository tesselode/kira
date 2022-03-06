mod handle;

pub use handle::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Key;

use super::scene::SpatialSceneId;

pub(crate) struct Emitter {
	shared: Arc<EmitterShared>,
}

impl Emitter {
	pub(crate) fn new() -> Self {
		Self {
			shared: Arc::new(EmitterShared::new()),
		}
	}

	pub(crate) fn shared(&self) -> Arc<EmitterShared> {
		self.shared.clone()
	}
}

/// A unique identifier for an emitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmitterId {
	pub(crate) key: Key,
	pub(crate) scene_id: SpatialSceneId,
}

impl EmitterId {
	/// Returns the ID of the spatial scene this emitter belongs to.
	pub fn scene(&self) -> SpatialSceneId {
		self.scene_id
	}
}

pub(crate) struct EmitterShared {
	removed: AtomicBool,
}

impl EmitterShared {
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
