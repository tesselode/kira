use std::sync::Arc;

use glam::{Quat, Vec3};

use crate::{
	manager::backend::resources::ResourceStorage,
	sound::{Sound, SoundData},
	tween::{Parameter, Value},
};

use super::{
	command_writers_and_readers,
	sound::{SpatialScene, SpatialSceneShared},
	SpatialSceneHandle,
};

/// Settings for a spatial scene.
pub struct SpatialSceneSettings {
	/// The maximum number of emitters that can be in the scene at once.
	pub emitter_capacity: u16,
	/// The position of the listener in the spatial scene.
	pub listener_position: Value<mint::Vector3<f32>>,
	/// The rotation of the listener in the spatial scene.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	pub listener_orientation: Value<mint::Quaternion<f32>>,
}

impl SpatialSceneSettings {
	/// Creates a new [`SpatialSceneSettings`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			emitter_capacity: 128,
			listener_position: Value::Fixed(mint::Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			}),
			listener_orientation: Value::Fixed(mint::Quaternion {
				v: mint::Vector3 {
					x: 0.0,
					y: 0.0,
					z: 0.0,
				},
				s: 1.0,
			}),
		}
	}

	/// Sets the maximum number of emitters that can be in the scene at once.
	#[must_use = "This method consumes self and returns a modified SpatialSceneSettings, so the return value should be used"]
	pub fn emitter_capacity(self, emitter_capacity: u16) -> Self {
		Self {
			emitter_capacity,
			..self
		}
	}

	/// Sets the position of the listener in the spatial scene.
	#[must_use = "This method consumes self and returns a modified SpatialSceneSettings, so the return value should be used"]
	pub fn listener_position(
		self,
		listener_position: impl Into<Value<mint::Vector3<f32>>>,
	) -> Self {
		Self {
			listener_position: listener_position.into(),
			..self
		}
	}

	/// Sets the rotation of the listener in the spatial scene.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	#[must_use = "This method consumes self and returns a modified SpatialSceneSettings, so the return value should be used"]
	pub fn listener_orientation(
		self,
		listener_orientation: impl Into<Value<mint::Quaternion<f32>>>,
	) -> Self {
		Self {
			listener_orientation: listener_orientation.into(),
			..self
		}
	}
}

impl Default for SpatialSceneSettings {
	fn default() -> Self {
		Self::new()
	}
}

impl SoundData for SpatialSceneSettings {
	type Error = ();

	type Handle = SpatialSceneHandle;

	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let shared = Arc::new(SpatialSceneShared::new());
		let (command_writers, command_readers) = command_writers_and_readers();
		let (emitters, emitter_controller) = ResourceStorage::new(self.emitter_capacity);
		Ok((
			Box::new(SpatialScene {
				shared: shared.clone(),
				command_readers,
				listener_position: Parameter::new(self.listener_position.to_(), Vec3::ZERO),
				listener_orientation: Parameter::new(
					self.listener_orientation.to_(),
					Quat::IDENTITY,
				),
				emitters,
			}),
			SpatialSceneHandle {
				shared,
				emitter_controller,
				command_writers,
			},
		))
	}
}
