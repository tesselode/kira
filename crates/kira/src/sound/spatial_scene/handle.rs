use std::sync::Arc;

use crate::{
	command::ValueChangeCommand,
	manager::backend::resources::ResourceController,
	tween::{Tween, Value},
	ResourceLimitReached,
};

use super::{
	emitter, sound::SpatialSceneShared, CommandWriters, Emitter, EmitterHandle, EmitterSettings,
};

/// Controls a spatial scene.
///
/// When a [`SpatialSceneHandle`] is dropped, the corresponding
/// spatial scene will be removed.
#[derive(Debug)]
pub struct SpatialSceneHandle {
	pub(crate) shared: Arc<SpatialSceneShared>,
	pub(crate) emitter_controller: ResourceController<Emitter>,
	pub(crate) command_writers: CommandWriters,
}

impl SpatialSceneHandle {
	/// Sets the location of the listener in the spatial scene.
	pub fn set_listener_position(
		&mut self,
		position: impl Into<Value<mint::Vector3<f32>>>,
		tween: Tween,
	) {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.command_writers
			.set_listener_position
			.write(ValueChangeCommand {
				target: position.to_(),
				tween,
			})
	}

	/// Sets the rotation of the listener in the spatial scene.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	pub fn set_listener_orientation(
		&mut self,
		orientation: impl Into<Value<mint::Quaternion<f32>>>,
		tween: Tween,
	) {
		let orientation: Value<mint::Quaternion<f32>> = orientation.into();
		self.command_writers
			.set_listener_orientation
			.write(ValueChangeCommand {
				target: orientation.to_(),
				tween,
			})
	}

	/// Adds an emitter to the scene.
	pub fn add_emitter(
		&mut self,
		position: impl Into<Value<mint::Vector3<f32>>>,
		settings: EmitterSettings,
	) -> Result<EmitterHandle, ResourceLimitReached> {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.add_emitter_inner(position.to_(), settings)
	}

	/// Returns the number of emitters in the scene.
	#[must_use]
	pub fn num_emitters(&self) -> u16 {
		self.emitter_controller.len()
	}

	fn add_emitter_inner(
		&mut self,
		position: Value<glam::Vec3>,
		settings: EmitterSettings,
	) -> Result<EmitterHandle, ResourceLimitReached> {
		let key = self.emitter_controller.try_reserve()?;
		let (command_writers, command_readers) = emitter::command_writers_and_readers();
		let (emitter, sound_controller) = Emitter::new(command_readers, position, settings);
		let handle = EmitterHandle {
			shared: emitter.shared(),
			command_writers,
			sound_controller,
		};
		self.emitter_controller.insert_with_key(key, emitter);
		Ok(handle)
	}
}

impl Drop for SpatialSceneHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
