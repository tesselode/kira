mod handle;
mod settings;

pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::{Arena, Key};

use crate::{dsp::Frame, track::TrackId};

use super::{emitter::Emitter, scene::SpatialSceneId};

pub(crate) struct Listener {
	shared: Arc<ListenerShared>,
	track: TrackId,
}

impl Listener {
	pub fn new(settings: ListenerSettings) -> Self {
		Self {
			shared: Arc::new(ListenerShared::new()),
			track: settings.track,
		}
	}

	pub fn shared(&self) -> Arc<ListenerShared> {
		self.shared.clone()
	}

	pub fn track(&self) -> TrackId {
		self.track
	}

	pub fn process(&mut self, emitters: &Arena<Emitter>) -> Frame {
		let mut output = Frame::ZERO;
		for (_, emitter) in emitters {
			output += emitter.input();
		}
		output
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
