//! Receives audio in a 3D space.

mod handle;
mod settings;

pub use handle::*;
pub use settings::*;

use std::{
	f32::consts::FRAC_PI_8,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

use crate::arena::{Arena, Key};
use glam::{Quat, Vec3};

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::ValueChangeCommand,
	command_writers_and_readers,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	track::TrackId,
	tween::{Parameter, Tweenable, Value},
	Volume,
};

use super::{emitter::Emitter, scene::SpatialSceneId};

const EAR_DISTANCE: f32 = 0.1;
const EAR_ANGLE_FROM_HEAD: f32 = FRAC_PI_8;
const MIN_EAR_AMPLITUDE: f32 = 0.5;

pub(crate) struct Listener {
	shared: Arc<ListenerShared>,
	position: Parameter<Vec3>,
	orientation: Parameter<Quat>,
	track: TrackId,
	command_readers: CommandReaders,
}

impl Listener {
	pub fn new(
		position: Value<Vec3>,
		orientation: Value<Quat>,
		settings: ListenerSettings,
		command_readers: CommandReaders,
	) -> Self {
		Self {
			shared: Arc::new(ListenerShared::new()),
			position: Parameter::new(position, Vec3::ZERO),
			orientation: Parameter::new(orientation, Quat::IDENTITY),
			track: settings.track,
			command_readers,
		}
	}

	pub fn shared(&self) -> Arc<ListenerShared> {
		self.shared.clone()
	}

	pub fn track(&self) -> TrackId {
		self.track
	}

	pub fn on_start_processing(&mut self) {
		self.position
			.read_commands(&mut self.command_readers.position_change);
		self.orientation
			.read_commands(&mut self.command_readers.orientation_change);
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		emitters: &Arena<Emitter>,
	) -> Frame {
		self.position
			.update(dt, clock_info_provider, modulator_value_provider);
		self.orientation
			.update(dt, clock_info_provider, modulator_value_provider);
		let mut output = Frame::ZERO;
		for (_, emitter) in emitters {
			let mut emitter_output = emitter.output();
			// attenuate volume
			if let Some(attenuation_function) = emitter.attenuation_function() {
				let distance = (emitter.position() - self.position.value()).length();
				let relative_distance = emitter.distances().relative_distance(distance);
				let relative_volume =
					attenuation_function.apply((1.0 - relative_distance).into()) as f32;
				let amplitude = Tweenable::interpolate(
					Volume::Decibels(Volume::MIN_DECIBELS),
					Volume::Decibels(0.0),
					relative_volume.into(),
				)
				.as_amplitude() as f32;
				emitter_output *= amplitude;
			}
			// apply spatialization
			if emitter.enable_spatialization() {
				emitter_output = emitter_output.as_mono();
				let (left_ear_position, right_ear_position) = self.ear_positions();
				let (left_ear_direction, right_ear_direction) = self.ear_directions();
				let emitter_direction_relative_to_left_ear =
					(emitter.position() - left_ear_position).normalize_or_zero();
				let emitter_direction_relative_to_right_ear =
					(emitter.position() - right_ear_position).normalize_or_zero();
				let left_ear_volume =
					(left_ear_direction.dot(emitter_direction_relative_to_left_ear) + 1.0) / 2.0;
				let right_ear_volume =
					(right_ear_direction.dot(emitter_direction_relative_to_right_ear) + 1.0) / 2.0;
				emitter_output.left *=
					MIN_EAR_AMPLITUDE + (1.0 - MIN_EAR_AMPLITUDE) * left_ear_volume;
				emitter_output.right *=
					MIN_EAR_AMPLITUDE + (1.0 - MIN_EAR_AMPLITUDE) * right_ear_volume;
			}
			output += emitter_output;
		}
		output
	}

	fn ear_positions(&self) -> (Vec3, Vec3) {
		let position = self.position.value();
		let orientation = self.orientation.value();
		let left = position + orientation * (Vec3::NEG_X * EAR_DISTANCE);
		let right = position + orientation * (Vec3::X * EAR_DISTANCE);
		(left, right)
	}

	fn ear_directions(&self) -> (Vec3, Vec3) {
		let left_ear_direction_relative_to_head =
			Quat::from_rotation_y(-EAR_ANGLE_FROM_HEAD) * Vec3::NEG_X;
		let right_ear_direction_relative_to_head =
			Quat::from_rotation_y(EAR_ANGLE_FROM_HEAD) * Vec3::X;
		let orientation = self.orientation.value();
		let left = orientation * left_ear_direction_relative_to_head;
		let right = orientation * right_ear_direction_relative_to_head;
		(left, right)
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

command_writers_and_readers!(
	pub(crate) struct {
		position_change: ValueChangeCommand<Vec3>,
		orientation_change: ValueChangeCommand<Quat>
	}
);
