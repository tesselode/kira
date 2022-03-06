use std::sync::Arc;

use crate::manager::command::producer::CommandProducer;

use super::{SpatialSceneId, SpatialSceneShared};

/// Controls a spatial scene.
///
/// When a [`SpatialSceneHandle`] is dropped, the corresponding
/// spatial scene will be removed.
pub struct SpatialSceneHandle {
	pub(crate) id: SpatialSceneId,
	pub(crate) shared: Arc<SpatialSceneShared>,
	pub(crate) command_producer: CommandProducer,
}

impl SpatialSceneHandle {
	/// Returns the unique identifier for the spatial scene.
	pub fn id(&self) -> SpatialSceneId {
		self.id
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
