//! Receives audio in a 3D space.

mod handle;

pub use handle::*;

use std::{
	f32::consts::FRAC_PI_8,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

use crate::manager::backend::resources::spatial_scenes::SpatialScenes;
use glam::{Quat, Vec3};

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::read_commands_into_parameters,
	command::ValueChangeCommand,
	command_writers_and_readers,
	frame::Frame,
	modulator::value_provider::ModulatorValueProvider,
	tween::{Parameter, Tweenable, Value},
	Volume,
};

use super::scene::SpatialSceneId;

const EAR_DISTANCE: f32 = 0.1;
const EAR_ANGLE_FROM_HEAD: f32 = FRAC_PI_8;
const MIN_EAR_AMPLITUDE: f32 = 0.5;

pub(crate) struct Listener {
	spatial_scene_id: SpatialSceneId,
	command_readers: CommandReaders,
	shared: Arc<ListenerShared>,
	position: Parameter<Vec3>,
	orientation: Parameter<Quat>,
}

impl Listener {
	#[must_use]
	pub fn new(
		spatial_scene_id: SpatialSceneId,
		command_readers: CommandReaders,
		position: Value<Vec3>,
		orientation: Value<Quat>,
	) -> Self {
		Self {
			spatial_scene_id,
			command_readers,
			shared: Arc::new(ListenerShared::new()),
			position: Parameter::new(position, Vec3::ZERO),
			orientation: Parameter::new(orientation, Quat::IDENTITY),
		}
	}

	#[must_use]
	pub fn shared(&self) -> Arc<ListenerShared> {
		self.shared.clone()
	}

	pub fn finished(&self) -> bool {
		self.shared.is_marked_for_removal()
	}

	pub fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, position, orientation);
	}

	#[must_use]
	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		spatial_scenes: &SpatialScenes,
	) -> Frame {
		self.position
			.update(dt, clock_info_provider, modulator_value_provider);
		self.orientation
			.update(dt, clock_info_provider, modulator_value_provider);
		let Some(spatial_scene) = spatial_scenes.get(self.spatial_scene_id) else {
			return Frame::ZERO;
		};
		let mut output = Frame::ZERO;
		for (_, emitter) in spatial_scene.emitters() {
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

	#[must_use]
	fn ear_positions(&self) -> (Vec3, Vec3) {
		let position = self.position.value();
		let orientation = self.orientation.value();
		let left = position + orientation * (Vec3::NEG_X * EAR_DISTANCE);
		let right = position + orientation * (Vec3::X * EAR_DISTANCE);
		(left, right)
	}

	#[must_use]
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

#[derive(Debug)]
pub(crate) struct ListenerShared {
	removed: AtomicBool,
}

impl ListenerShared {
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
	set_orientation: ValueChangeCommand<Quat>,
}
