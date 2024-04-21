//! A 3D space that audio travels through.

mod handle;
mod settings;

pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use crate::{arena::Key, manager::backend::resources::ResourceStorage};

use crate::{
	clock::clock_info::ClockInfoProvider, manager::backend::resources::mixer::Mixer,
	modulator::value_provider::ModulatorValueProvider,
};

use super::{
	emitter::{Emitter, EmitterId},
	listener::Listener,
};

pub(crate) struct SpatialScene {
	emitters: ResourceStorage<Emitter>,
	listeners: ResourceStorage<Listener>,
	shared: Arc<SpatialSceneShared>,
}

impl SpatialScene {
	pub fn new(id: SpatialSceneId, settings: SpatialSceneSettings) -> (Self, SpatialSceneHandle) {
		let (emitters, emitter_controller) = ResourceStorage::new(settings.emitter_capacity);
		let (listeners, listener_controller) = ResourceStorage::new(settings.listener_capacity);
		let shared = Arc::new(SpatialSceneShared::new());
		(
			Self {
				emitters,
				listeners,
				shared: shared.clone(),
			},
			SpatialSceneHandle {
				id,
				shared,
				emitter_controller,
				listener_controller,
			},
		)
	}

	pub fn shared(&self) -> Arc<SpatialSceneShared> {
		self.shared.clone()
	}

	pub fn emitter_mut(&mut self, id: EmitterId) -> Option<&mut Emitter> {
		self.emitters.get_mut(id.key)
	}

	pub fn on_start_processing(&mut self) {
		self.emitters.remove_and_add(|emitter| emitter.finished());
		self.listeners
			.remove_and_add(|listener| listener.shared().is_marked_for_removal());
		for (_, listener) in &mut self.listeners {
			listener.on_start_processing();
		}
		for (_, emitter) in &mut self.emitters {
			emitter.on_start_processing();
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
					&self.emitters.resources,
				));
			}
		}
		for (_, emitter) in &mut self.emitters {
			emitter.after_process();
		}
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
