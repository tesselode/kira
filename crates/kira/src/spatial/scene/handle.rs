use glam::{Quat, Vec3};

use std::sync::Arc;

use crate::manager::backend::resources::ResourceController;

use crate::ResourceLimitReached;
use crate::{
	spatial::{
		emitter::{self, Emitter, EmitterHandle, EmitterId, EmitterSettings},
		listener::{self, Listener, ListenerHandle, ListenerSettings},
	},
	tween::Value,
};

use super::{SpatialSceneId, SpatialSceneShared};

/// Controls a spatial scene.
///
/// When a [`SpatialSceneHandle`] is dropped, the corresponding
/// spatial scene will be removed.
pub struct SpatialSceneHandle {
	pub(crate) id: SpatialSceneId,
	pub(crate) shared: Arc<SpatialSceneShared>,
	pub(crate) emitter_controller: ResourceController<Emitter>,
	pub(crate) listener_controller: ResourceController<Listener>,
}

impl SpatialSceneHandle {
	/// Returns the unique identifier for the spatial scene.
	#[must_use]
	pub fn id(&self) -> SpatialSceneId {
		self.id
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

	/// Adds a listener to the scene.
	///
	/// An unrotated listener should face in the negative Z direction with
	/// positive X to the right and positive Y up.
	pub fn add_listener(
		&mut self,
		position: impl Into<Value<mint::Vector3<f32>>>,
		orientation: impl Into<Value<mint::Quaternion<f32>>>,
		settings: ListenerSettings,
	) -> Result<ListenerHandle, ResourceLimitReached> {
		let position: Value<mint::Vector3<f32>> = position.into();
		let orientation: Value<mint::Quaternion<f32>> = orientation.into();
		self.add_listener_inner(position.to_(), orientation.to_(), settings)
	}

	/// Returns the number of emitters in the scene.
	#[must_use]
	pub fn num_emitters(&self) -> u16 {
		self.emitter_controller.len()
	}

	/// Returns the number of listeners in the scene.
	#[must_use]
	pub fn num_listeners(&self) -> u16 {
		self.listener_controller.len()
	}

	fn add_emitter_inner(
		&mut self,
		position: Value<glam::Vec3>,
		settings: EmitterSettings,
	) -> Result<EmitterHandle, ResourceLimitReached> {
		let key = self.emitter_controller.try_reserve()?;
		let id = EmitterId {
			key,
			scene_id: self.id,
		};
		let (command_writers, command_readers) = emitter::command_writers_and_readers();
		let emitter = Emitter::new(command_readers, position, settings);
		let handle = EmitterHandle {
			id,
			shared: emitter.shared(),
			command_writers,
		};
		self.emitter_controller.insert_with_key(key, emitter);
		Ok(handle)
	}

	fn add_listener_inner(
		&mut self,
		position: Value<Vec3>,
		orientation: Value<Quat>,
		settings: ListenerSettings,
	) -> Result<ListenerHandle, ResourceLimitReached> {
		let (command_writers, command_readers) = listener::command_writers_and_readers();
		let listener = Listener::new(command_readers, position, orientation, settings);
		let handle = ListenerHandle {
			shared: listener.shared(),
			command_writers,
		};
		self.listener_controller.insert(listener)?;
		Ok(handle)
	}
}

impl Drop for SpatialSceneHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&SpatialSceneHandle> for SpatialSceneId {
	fn from(handle: &SpatialSceneHandle) -> Self {
		handle.id()
	}
}
