use std::sync::Arc;

use crate::{
	manager::command::{producer::CommandProducer, Command, SpatialSceneCommand},
	CommandError,
};

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

	pub fn set_position(
		&mut self,
		position: impl Into<mint::Vector3<f32>>,
	) -> Result<(), CommandError> {
		let position: mint::Vector3<f32> = position.into();
		self.command_producer.push(Command::SpatialScene(
			SpatialSceneCommand::SetEmitterPosition(self.id, position.into()),
		))
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
