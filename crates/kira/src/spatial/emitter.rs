//! Produces audio in a 3D space.

mod distances;
mod handle;
mod settings;

pub use distances::*;
pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use crate::arena::Key;
use glam::Vec3;

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::read_commands_into_parameters,
	command::ValueChangeCommand,
	command_writers_and_readers,
	frame::Frame,
	modulator::value_provider::ModulatorValueProvider,
	tween::{Easing, Parameter, Value},
};

use super::scene::SpatialSceneId;

pub(crate) struct Emitter {
	command_readers: CommandReaders,
	shared: Arc<EmitterShared>,
	position: Parameter<Vec3>,
	distances: EmitterDistances,
	attenuation_function: Option<Easing>,
	enable_spatialization: bool,
	persist_until_sounds_finish: bool,
	input: Frame,
	used_this_frame: bool,
	finished: bool,
}

impl Emitter {
	#[must_use]
	pub fn new(
		command_readers: CommandReaders,
		position: Value<Vec3>,
		settings: EmitterSettings,
	) -> Self {
		Self {
			command_readers,
			shared: Arc::new(EmitterShared::new()),
			position: Parameter::new(position, Vec3::ZERO),
			distances: settings.distances,
			attenuation_function: settings.attenuation_function,
			enable_spatialization: settings.enable_spatialization,
			persist_until_sounds_finish: settings.persist_until_sounds_finish,
			input: Frame::ZERO,
			used_this_frame: false,
			finished: false,
		}
	}

	#[must_use]
	pub fn output(&self) -> Frame {
		self.input
	}

	#[must_use]
	pub fn shared(&self) -> Arc<EmitterShared> {
		self.shared.clone()
	}

	#[must_use]
	pub fn position(&self) -> Vec3 {
		self.position.value()
	}

	#[must_use]
	pub fn distances(&self) -> EmitterDistances {
		self.distances
	}

	#[must_use]
	pub fn attenuation_function(&self) -> Option<Easing> {
		self.attenuation_function
	}

	#[must_use]
	pub fn enable_spatialization(&self) -> bool {
		self.enable_spatialization
	}

	#[must_use]
	pub fn finished(&self) -> bool {
		self.finished
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
		self.used_this_frame = true;
	}

	pub fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, position);
	}

	pub fn after_process(&mut self) {
		if self.should_be_finished() {
			self.finished = true;
		}
		self.input = Frame::ZERO;
		self.used_this_frame = false;
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

	#[must_use]
	fn should_be_finished(&self) -> bool {
		if !self.shared.is_marked_for_removal() {
			return false;
		}
		if self.persist_until_sounds_finish && self.used_this_frame {
			return false;
		}
		true
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
	#[must_use]
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

command_writers_and_readers! {
	set_position: ValueChangeCommand<Vec3>,
}
