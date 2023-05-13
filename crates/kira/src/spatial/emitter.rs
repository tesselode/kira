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
	modulator::value_provider::ModulatorValueProvider,
	tween::{Easing, Parameter, Tween, Value},
};

use super::scene::SpatialSceneId;

pub(crate) struct Emitter {
	shared: Arc<EmitterShared>,
	position: Parameter<Vec3>,
	distances: EmitterDistances,
	attenuation_function: Option<Easing>,
	enable_spatialization: bool,
	input: Frame,
}

impl Emitter {
	pub fn new(position: Value<Vec3>, settings: EmitterSettings) -> Self {
		Self {
			shared: Arc::new(EmitterShared::new()),
			position: Parameter::new(position, Vec3::ZERO),
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

	pub fn set_position(&mut self, position: Value<Vec3>, tween: Tween) {
		self.position.set(position, tween);
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn reset_input(&mut self) {
		self.input = Frame::ZERO;
	}

	pub fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		self.position
			.update(dt, clock_info_provider, modulator_value_provider);
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
