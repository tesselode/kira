use std::sync::Arc;

use crate::manager::command::producer::CommandProducer;

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
