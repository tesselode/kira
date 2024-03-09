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

use atomic_arena::Key;
use glam::Vec3;

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::ValueChangeCommand,
	command_writers_and_readers,
	dsp::Frame,
	manager::backend::Renderer,
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
	persist_until_sounds_finish: bool,
	input: Vec<Frame>,
	used_this_frame: bool,
	finished: bool,
	command_readers: CommandReaders,
}

impl Emitter {
	pub fn new(
		position: Value<Vec3>,
		settings: EmitterSettings,
		command_readers: CommandReaders,
	) -> Self {
		Self {
			shared: Arc::new(EmitterShared::new()),
			position: Parameter::new(position, Vec3::ZERO),
			distances: settings.distances,
			attenuation_function: settings.attenuation_function,
			enable_spatialization: settings.enable_spatialization,
			persist_until_sounds_finish: settings.persist_until_sounds_finish,
			input: vec![Frame::ZERO; Renderer::INTERNAL_BUFFER_SIZE],
			used_this_frame: false,
			finished: false,
			command_readers,
		}
	}

	pub fn output(&self, frame_index: usize) -> Frame {
		self.input[frame_index]
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

	pub fn finished(&self) -> bool {
		self.finished
	}

	pub fn on_start_processing(&mut self) {
		self.position
			.read_commands(&mut self.command_readers.position_change);
	}

	pub fn set_position(&mut self, position: Value<Vec3>, tween: Tween) {
		self.position.set(position, tween);
	}

	pub fn add_input(&mut self, frame_index: usize, input: Frame) {
		self.input[frame_index] += input;
		self.used_this_frame = true;
	}

	pub fn after_process(&mut self, frame_index: usize) {
		if self.should_be_finished() {
			self.finished = true;
		}
		self.input[frame_index] = Frame::ZERO;
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

command_writers_and_readers!(
	pub(crate) struct {
		position_change: ValueChangeCommand<Vec3>
	}
);
