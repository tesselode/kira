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

use crate::{
	manager::backend::resources::{ResourceController, ResourceStorage},
	sound::Sound,
};
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

pub(crate) struct Emitter {
	command_readers: CommandReaders,
	shared: Arc<EmitterShared>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	position: Parameter<Vec3>,
	distances: EmitterDistances,
	attenuation_function: Option<Easing>,
	enable_spatialization: bool,
	persist_until_sounds_finish: bool,
	output: Frame,
}

impl Emitter {
	#[must_use]
	pub fn new(
		command_readers: CommandReaders,
		position: Value<Vec3>,
		settings: EmitterSettings,
	) -> (Self, ResourceController<Box<dyn Sound>>) {
		let (sounds, sound_controller) = ResourceStorage::new(settings.sound_capacity);
		(
			Self {
				command_readers,
				shared: Arc::new(EmitterShared::new()),
				sounds,
				position: Parameter::new(position, Vec3::ZERO),
				distances: settings.distances,
				attenuation_function: settings.attenuation_function,
				enable_spatialization: settings.enable_spatialization,
				persist_until_sounds_finish: settings.persist_until_sounds_finish,
				output: Frame::ZERO,
			},
			sound_controller,
		)
	}

	#[must_use]
	pub fn output(&self) -> Frame {
		self.output
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
		if self.persist_until_sounds_finish {
			self.shared().is_marked_for_removal() && self.sounds.is_empty()
		} else {
			self.shared().is_marked_for_removal()
		}
	}

	pub fn on_start_processing(&mut self) {
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		read_commands_into_parameters!(self, position);
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		self.position
			.update(dt, clock_info_provider, modulator_value_provider);
		self.output = Frame::ZERO;
		for (_, sound) in &mut self.sounds {
			self.output += sound.process(dt, clock_info_provider, modulator_value_provider);
		}
	}
}

#[derive(Debug)]
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
