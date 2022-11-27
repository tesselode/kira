mod distances;
mod handle;
mod settings;

pub use distances::*;
use glam::Vec3;
pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Key;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	tween::{Easing, Tween, Tweener},
};

use super::scene::SpatialSceneId;

pub(crate) struct Emitter {
	shared: Arc<EmitterShared>,
	position: Tweener<Vec3>,
	distances: EmitterDistances,
	attenuation_function: Option<Easing>,
	enable_spatialization: bool,
	input: Frame,
}

impl Emitter {
	pub fn new(position: Vec3, settings: EmitterSettings) -> Self {
		Self {
			shared: Arc::new(EmitterShared::new()),
			position: Tweener::new(position),
			distances: settings.distances,
			attenuation_function: settings.attenuation_function,
			enable_spatialization: settings.enable_spatialization,
			input: Frame::ZERO,
		}
	}

	pub fn output(&self) -> Frame {
		self.input
	}

	pub fn shared(&self) -> Arc<EmitterShared> {
		self.shared.clone()
	}

	pub fn position(&self) -> Vec3 {
		self.position.value()
	}

	pub fn distances(&self) -> EmitterDistances {
		self.distances
	}

	pub fn attenuation_function(&self) -> Option<Easing> {
		self.attenuation_function
	}

	pub fn enable_spatialization(&self) -> bool {
		self.enable_spatialization
	}

	pub fn set_position(&mut self, position: Vec3, tween: Tween) {
		self.position.set(position, tween);
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn reset_input(&mut self) {
		self.input = Frame::ZERO;
	}

	pub fn update(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) {
		self.position.update(dt, clock_info_provider);
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
