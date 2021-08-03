use std::sync::Arc;

use super::{AudioStreamId, AudioStreamShared};

pub struct AudioStreamHandle {
	pub(crate) id: AudioStreamId,
	pub(crate) shared: Arc<AudioStreamShared>,
}

impl AudioStreamHandle {
	pub fn id(&self) -> AudioStreamId {
		self.id
	}
}

impl Drop for AudioStreamHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
