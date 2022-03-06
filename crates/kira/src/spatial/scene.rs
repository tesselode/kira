mod handle;
mod settings;

pub use handle::*;
use ringbuf::{Consumer, Producer, RingBuffer};
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::{Arena, Controller, Key};

use super::emitter::{Emitter, EmitterId};

pub(crate) struct SpatialScene {
	emitters: Arena<Emitter>,
	unused_emitter_producer: Producer<Emitter>,
	shared: Arc<SpatialSceneShared>,
}

impl SpatialScene {
	pub fn new(settings: SpatialSceneSettings) -> (Self, Consumer<Emitter>) {
		let (unused_emitter_producer, unused_emitter_consumer) =
			RingBuffer::new(settings.emitter_capacity).split();
		(
			Self {
				emitters: Arena::new(settings.emitter_capacity),
				unused_emitter_producer,
				shared: Arc::new(SpatialSceneShared::new()),
			},
			unused_emitter_consumer,
		)
	}

	pub fn shared(&self) -> Arc<SpatialSceneShared> {
		self.shared.clone()
	}

	pub fn emitter_controller(&self) -> Controller {
		self.emitters.controller()
	}

	pub fn on_start_processing(&mut self) {
		if self.unused_emitter_producer.is_full() {
			return;
		}
		for (_, emitter) in self
			.emitters
			.drain_filter(|emitter| emitter.shared().is_marked_for_removal())
		{
			if self.unused_emitter_producer.push(emitter).is_err() {
				panic!("Unused emitter producer is full")
			}
			if self.unused_emitter_producer.is_full() {
				return;
			}
		}
	}

	pub fn add_emitter(&mut self, id: EmitterId, emitter: Emitter) {
		self.emitters
			.insert_with_key(id.key, emitter)
			.expect("Emitter arena is full");
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
