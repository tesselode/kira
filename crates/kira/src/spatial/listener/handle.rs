use std::sync::Arc;

use crate::{
	manager::command::{producer::CommandProducer, Command, SpatialSceneCommand},
	tween::{Tween, Value},
	CommandError,
};

use super::{ListenerId, ListenerShared};

/// Controls a listener.
///
/// When a [`ListenerHandle`] is dropped, the corresponding
/// listener will be removed.
pub struct ListenerHandle {
	pub(crate) id: ListenerId,
	pub(crate) shared: Arc<ListenerShared>,
	pub(crate) command_producer: CommandProducer,
}

impl ListenerHandle {
	/// Returns the unique identifier for the listener.
	pub fn id(&self) -> ListenerId {
		self.id
	}

	/// Sets the location of the listener in the spatial scene.
	pub fn set_position(
		&mut self,
		position: impl Into<Value<mint::Vector3<f32>>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		let position: Value<mint::Vector3<f32>> = position.into();
		self.command_producer.push(Command::SpatialScene(
			SpatialSceneCommand::SetListenerPosition(self.id, position.to_(), tween),
		))
	}

	/// Sets the rotation of the listener.
	pub fn set_orientation(
		&mut self,
		orientation: impl Into<Value<mint::Quaternion<f32>>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		let orientation: Value<mint::Quaternion<f32>> = orientation.into();
		self.command_producer.push(Command::SpatialScene(
			SpatialSceneCommand::SetListenerOrientation(self.id, orientation.to_(), tween),
		))
	}
}

impl Drop for ListenerHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&ListenerHandle> for ListenerId {
	fn from(handle: &ListenerHandle) -> Self {
		handle.id()
	}
}
