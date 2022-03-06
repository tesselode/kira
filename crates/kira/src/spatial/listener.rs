mod handle;

pub use handle::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Key;

use super::scene::SpatialSceneId;

pub(crate) struct Listener {
	shared: Arc<ListenerShared>,
}

impl Listener {
	pub(crate) fn new() -> Self {
		Self {
			shared: Arc::new(ListenerShared::new()),
		}
	}

	pub(crate) fn shared(&self) -> Arc<ListenerShared> {
		self.shared.clone()
	}
}

/// A unique identifier for an listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId {
	pub(crate) key: Key,
	pub(crate) scene_id: SpatialSceneId,
}

impl ListenerId {
	/// Returns the ID of the spatial scene this listener belongs to.
	pub fn scene(&self) -> SpatialSceneId {
		self.scene_id
	}
}

pub(crate) struct ListenerShared {
	removed: AtomicBool,
}

impl ListenerShared {
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
