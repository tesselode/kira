mod handle;
mod settings;

use glam::{Quat, Vec3};
pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::{Arena, Key};

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	track::TrackId,
	tween::{Tween, Tweenable, Tweener},
	Volume,
};

use super::{emitter::Emitter, scene::SpatialSceneId};

const EAR_DISTANCE: f32 = 0.1;
const MIN_EAR_AMPLITUDE: f32 = 0.5;

pub(crate) struct Listener {
	shared: Arc<ListenerShared>,
	position: Tweener<Vec3>,
	orientation: Tweener<Quat>,
	track: TrackId,
}

impl Listener {
	pub fn new(position: Vec3, orientation: Quat, settings: ListenerSettings) -> Self {
		Self {
			shared: Arc::new(ListenerShared::new()),
			position: Tweener::new(position),
			orientation: Tweener::new(orientation),
			track: settings.track,
		}
	}

	pub fn shared(&self) -> Arc<ListenerShared> {
		self.shared.clone()
	}

	pub fn track(&self) -> TrackId {
		self.track
	}

	pub fn set_position(&mut self, position: Vec3, tween: Tween) {
		self.position.set(position, tween);
	}

	pub fn set_orientation(&mut self, orientation: Quat, tween: Tween) {
		self.orientation.set(orientation, tween);
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		emitters: &Arena<Emitter>,
	) -> Frame {
		self.position.update(dt, clock_info_provider);
		self.orientation.update(dt, clock_info_provider);
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
				let (left_ear_position, right_ear_position) = self.ear_positions();
				let orientation = self.orientation.value();
				let left_ear_direction = orientation * Vec3::NEG_X;
				let right_ear_direction = orientation * Vec3::X;
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
