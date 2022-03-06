mod error;

use std::sync::Arc;

use atomic_arena::Controller;
use ringbuf::Consumer;

use crate::{
	manager::command::{producer::CommandProducer, Command, SpatialSceneCommand},
	spatial::emitter::{Emitter, EmitterHandle, EmitterId},
};

use self::error::AddEmitterError;

use super::{SpatialSceneId, SpatialSceneShared};

/// Controls a spatial scene.
///
/// When a [`SpatialSceneHandle`] is dropped, the corresponding
/// spatial scene will be removed.
pub struct SpatialSceneHandle {
	pub(crate) id: SpatialSceneId,
	pub(crate) shared: Arc<SpatialSceneShared>,
	pub(crate) emitter_controller: Controller,
	pub(crate) unused_emitter_consumer: Consumer<Emitter>,
	pub(crate) command_producer: CommandProducer,
}

impl SpatialSceneHandle {
	/// Returns the unique identifier for the spatial scene.
	pub fn id(&self) -> SpatialSceneId {
		self.id
	}

	/// Adds an emitter to the scene.
	pub fn add_emitter(&mut self) -> Result<EmitterHandle, AddEmitterError> {
		while self.unused_emitter_consumer.pop().is_some() {}
		let id = EmitterId {
			key: self
				.emitter_controller
				.try_reserve()
				.map_err(|_| AddEmitterError::EmitterLimitReached)?,
			scene_id: self.id,
		};
		let emitter = Emitter::new();
		let handle = EmitterHandle {
			id,
			shared: emitter.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::SpatialScene(SpatialSceneCommand::AddEmitter(
				id, emitter,
			)))?;
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
