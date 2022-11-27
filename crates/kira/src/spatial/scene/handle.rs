mod error;

pub use error::*;
use glam::{Quat, Vec3};

use std::sync::Arc;

use atomic_arena::Controller;
use ringbuf::HeapConsumer;

use crate::{
	manager::command::{producer::CommandProducer, Command, SpatialSceneCommand},
	spatial::{
		emitter::{Emitter, EmitterHandle, EmitterId, EmitterSettings},
		listener::{Listener, ListenerHandle, ListenerId, ListenerSettings},
	},
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
	pub(crate) unused_emitter_consumer: HeapConsumer<Emitter>,
	pub(crate) listener_controller: Controller,
	pub(crate) unused_listener_consumer: HeapConsumer<Listener>,
	pub(crate) command_producer: CommandProducer,
}

impl SpatialSceneHandle {
	/// Returns the unique identifier for the spatial scene.
	pub fn id(&self) -> SpatialSceneId {
		self.id
	}

	/// Adds an emitter to the scene.
	pub fn add_emitter(
		&mut self,
		position: impl Into<mint::Vector3<f32>>,
		settings: EmitterSettings,
	) -> Result<EmitterHandle, AddEmitterError> {
		let position: mint::Vector3<f32> = position.into();
		self.add_emitter_inner(position.into(), settings)
	}

	/// Adds a listener to the scene.
	pub fn add_listener(
		&mut self,
		position: impl Into<mint::Vector3<f32>>,
		orientation: impl Into<mint::Quaternion<f32>>,
		settings: ListenerSettings,
	) -> Result<ListenerHandle, AddListenerError> {
		let position: mint::Vector3<f32> = position.into();
		let orientation: mint::Quaternion<f32> = orientation.into();
		self.add_listener_inner(position.into(), orientation.into(), settings)
	}

	fn add_emitter_inner(
		&mut self,
		position: glam::Vec3,
		settings: EmitterSettings,
	) -> Result<EmitterHandle, AddEmitterError> {
		while self.unused_emitter_consumer.pop().is_some() {}
		let id = EmitterId {
			key: self
				.emitter_controller
				.try_reserve()
				.map_err(|_| AddEmitterError::EmitterLimitReached)?,
			scene_id: self.id,
		};
		let emitter = Emitter::new(position, settings);
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

	fn add_listener_inner(
		&mut self,
		position: Vec3,
		orientation: Quat,
		settings: ListenerSettings,
	) -> Result<ListenerHandle, AddListenerError> {
		while self.unused_listener_consumer.pop().is_some() {}
		let id = ListenerId {
			key: self
				.listener_controller
				.try_reserve()
				.map_err(|_| AddListenerError::ListenerLimitReached)?,
			scene_id: self.id,
		};
		let listener = Listener::new(position, orientation, settings);
		let handle = ListenerHandle {
			id,
			shared: listener.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::SpatialScene(SpatialSceneCommand::AddListener(
				id, listener,
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
