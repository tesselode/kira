use std::sync::Arc;

use crate::manager::backend::resources::ResourceController;

use crate::ResourceLimitReached;
use crate::{
	spatial::emitter::{self, Emitter, EmitterHandle, EmitterSettings},
	tween::Value,
};

use super::{SpatialSceneId, SpatialSceneShared};

/// Controls a spatial scene.
///
/// When a [`SpatialSceneHandle`] is dropped, the corresponding
/// spatial scene will be removed.
#[derive(Debug)]
pub struct SpatialSceneHandle {
	pub(crate) id: SpatialSceneId,
	pub(crate) shared: Arc<SpatialSceneShared>,
	pub(crate) emitter_controller: ResourceController<Emitter>,
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

impl From<&SpatialSceneHandle> for SpatialSceneId {
	fn from(handle: &SpatialSceneHandle) -> Self {
		handle.id()
	}
}
