use std::sync::Arc;

use crate::manager::command::producer::CommandProducer;

use super::{EmitterId, EmitterShared};

/// Controls a emitter.
///
/// When a [`EmitterHandle`] is dropped, the corresponding
/// emitter will be removed.
pub struct EmitterHandle {
	pub(crate) id: EmitterId,
	pub(crate) shared: Arc<EmitterShared>,
	pub(crate) command_producer: CommandProducer,
}

impl EmitterHandle {
	/// Returns the unique identifier for the emitter.
	pub fn id(&self) -> EmitterId {
		self.id
	}
}

impl Drop for EmitterHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}

impl From<&EmitterHandle> for EmitterId {
	fn from(handle: &EmitterHandle) -> Self {
		handle.id()
	}
}
