use std::sync::Arc;

use crate::{
	manager::command::{producer::CommandProducer, Command, SpatialSceneCommand},
	math::Vec3,
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

	pub fn set_position(&mut self, position: Vec3) -> Result<(), CommandError> {
		self.command_producer.push(Command::SpatialScene(
			SpatialSceneCommand::SetListenerPosition(self.id, position),
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
