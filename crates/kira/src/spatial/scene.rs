//! A 3D space that audio travels through.

mod handle;
mod settings;

pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::{Arena, Controller, Key};
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};

use crate::{
	clock::clock_info::ClockInfoProvider, manager::backend::resources::mixer::Mixer,
	modulator::value_provider::ModulatorValueProvider,
};

use super::{
	emitter::{Emitter, EmitterId},
	listener::{Listener, ListenerId},
};

pub(crate) struct SpatialScene {
	emitters: Arena<Emitter>,
	unused_emitter_producer: HeapProducer<Emitter>,
	listeners: Arena<Listener>,
	unused_listener_producer: HeapProducer<Listener>,
	shared: Arc<SpatialSceneShared>,
}

impl SpatialScene {
	pub fn new(
		settings: SpatialSceneSettings,
	) -> (Self, HeapConsumer<Emitter>, HeapConsumer<Listener>) {
		let (unused_emitter_producer, unused_emitter_consumer) =
			HeapRb::new(settings.emitter_capacity).split();
		let (unused_listener_producer, unused_listener_consumer) =
			HeapRb::new(settings.listener_capacity).split();
		(
			Self {
				emitters: Arena::new(settings.emitter_capacity),
				unused_emitter_producer,
				listeners: Arena::new(settings.listener_capacity),
				unused_listener_producer,
				shared: Arc::new(SpatialSceneShared::new()),
			},
			unused_emitter_consumer,
			unused_listener_consumer,
		)
	}

	pub fn shared(&self) -> Arc<SpatialSceneShared> {
		self.shared.clone()
	}

	pub fn emitter_controller(&self) -> Controller {
		self.emitters.controller()
	}

	pub fn listener_controller(&self) -> Controller {
		self.listeners.controller()
	}

	pub fn emitter_mut(&mut self, id: EmitterId) -> Option<&mut Emitter> {
		self.emitters.get_mut(id.key)
	}

	pub fn on_start_processing(&mut self) {
		self.remove_unused_emitters();
		self.remove_unused_listeners();
		for (_, listener) in &mut self.listeners {
			listener.on_start_processing();
		}
		for (_, emitter) in &mut self.emitters {
			emitter.on_start_processing();
		}
	}

	pub fn remove_unused_emitters(&mut self) {
		if self.unused_emitter_producer.is_full() {
			return;
		}
		for (_, emitter) in self.emitters.drain_filter(|emitter| emitter.finished()) {
			if self.unused_emitter_producer.push(emitter).is_err() {
				panic!("Unused emitter producer is full")
			}
			if self.unused_emitter_producer.is_full() {
				return;
			}
		}
	}

	pub fn remove_unused_listeners(&mut self) {
		if self.unused_listener_producer.is_full() {
			return;
		}
		for (_, listener) in self
			.listeners
			.drain_filter(|listener| listener.shared().is_marked_for_removal())
		{
			if self.unused_listener_producer.push(listener).is_err() {
				panic!("Unused listener producer is full")
			}
			if self.unused_listener_producer.is_full() {
				return;
			}
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		mixer: &mut Mixer,
	) {
		for (_, emitter) in &mut self.emitters {
			emitter.update(dt, clock_info_provider, modulator_value_provider);
		}
		for (_, listener) in &mut self.listeners {
			if let Some(track) = mixer.track_mut(listener.track()) {
				track.add_input(listener.process(
					dt,
					clock_info_provider,
					modulator_value_provider,
					&self.emitters,
				));
			}
		}
		for (_, emitter) in &mut self.emitters {
			emitter.after_process();
		}
	}

	pub fn add_emitter(&mut self, id: EmitterId, emitter: Emitter) {
		self.emitters
			.insert_with_key(id.key, emitter)
			.expect("Emitter arena is full");
	}

	pub fn add_listener(&mut self, id: ListenerId, listener: Listener) {
		self.listeners
			.insert_with_key(id.key, listener)
			.expect("Listener arena is full");
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
