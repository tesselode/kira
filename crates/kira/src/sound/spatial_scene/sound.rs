use std::{
	f32::consts::FRAC_PI_8,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

use glam::{Quat, Vec3};

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::read_commands_into_parameters,
	manager::backend::resources::ResourceStorage,
	modulator::value_provider::ModulatorValueProvider,
	sound::Sound,
	tween::{Parameter, Tweenable},
	Frame, Volume,
};

use super::{CommandReaders, Emitter};

const EAR_DISTANCE: f32 = 0.1;
const EAR_ANGLE_FROM_HEAD: f32 = FRAC_PI_8;
const MIN_EAR_AMPLITUDE: f32 = 0.5;

pub(crate) struct SpatialScene {
	pub(crate) shared: Arc<SpatialSceneShared>,
	pub(crate) command_readers: CommandReaders,
	pub(crate) listener_position: Parameter<Vec3>,
	pub(crate) listener_orientation: Parameter<Quat>,
	pub(crate) emitters: ResourceStorage<Emitter>,
}

impl SpatialScene {
	#[must_use]
	fn listener_ear_positions(&self) -> (Vec3, Vec3) {
		let position = self.listener_position.value();
		let orientation = self.listener_orientation.value();
		let left = position + orientation * (Vec3::NEG_X * EAR_DISTANCE);
		let right = position + orientation * (Vec3::X * EAR_DISTANCE);
		(left, right)
	}

	#[must_use]
	fn listener_ear_directions(&self) -> (Vec3, Vec3) {
		let left_ear_direction_relative_to_head =
			Quat::from_rotation_y(-EAR_ANGLE_FROM_HEAD) * Vec3::NEG_X;
		let right_ear_direction_relative_to_head =
			Quat::from_rotation_y(EAR_ANGLE_FROM_HEAD) * Vec3::X;
		let orientation = self.listener_orientation.value();
		let left = orientation * left_ear_direction_relative_to_head;
		let right = orientation * right_ear_direction_relative_to_head;
		(left, right)
	}
}

impl Sound for SpatialScene {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, listener_position, listener_orientation);
		self.emitters.remove_and_add(|emitter| emitter.finished());
		for (_, emitter) in &mut self.emitters {
			emitter.on_start_processing();
		}
	}

	fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		// process emitters
		for (_, emitter) in &mut self.emitters {
			emitter.process(dt, clock_info_provider, modulator_value_provider);
		}

		// update listener position and orientation
		self.listener_position
			.update(dt, clock_info_provider, modulator_value_provider);
		self.listener_orientation
			.update(dt, clock_info_provider, modulator_value_provider);

		// process listener
		let mut output = Frame::ZERO;
		for (_, emitter) in &self.emitters {
			let mut emitter_output = emitter.output();
			// attenuate volume
			if let Some(attenuation_function) = emitter.attenuation_function() {
				let distance = (emitter.position() - self.listener_position.value()).length();
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
				let (left_ear_position, right_ear_position) = self.listener_ear_positions();
				let (left_ear_direction, right_ear_direction) = self.listener_ear_directions();
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

	fn finished(&self) -> bool {
		self.shared.is_marked_for_removal()
	}
}

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
