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
	clock::clock_info::ClockInfoProvider, modulator::value_provider::ModulatorValueProvider,
};

use super::emitter::Emitter;

pub(crate) struct SpatialScene {
	emitters: ResourceStorage<Emitter>,
	shared: Arc<SpatialSceneShared>,
}

impl SpatialScene {
	#[must_use]
	pub fn new(id: SpatialSceneId, settings: SpatialSceneSettings) -> (Self, SpatialSceneHandle) {
		let (emitters, emitter_controller) = ResourceStorage::new(settings.emitter_capacity);
		let shared = Arc::new(SpatialSceneShared::new());
		(
			Self {
				emitters,
				shared: shared.clone(),
			},
			SpatialSceneHandle {
				id,
				shared,
				emitter_controller,
			},
		)
	}

	#[must_use]
	pub fn shared(&self) -> Arc<SpatialSceneShared> {
		self.shared.clone()
	}

	pub fn emitters(&self) -> crate::arena::iter::Iter<Emitter> {
		self.emitters.iter()
	}

	pub fn on_start_processing(&mut self) {
		self.emitters.remove_and_add(|emitter| emitter.finished());
		for (_, emitter) in &mut self.emitters {
			emitter.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		for (_, emitter) in &mut self.emitters {
			emitter.process(dt, clock_info_provider, modulator_value_provider);
		}
	}
}

/// A unique identifier for a spatial scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpatialSceneId(pub(crate) Key);

#[derive(Debug)]
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
