use std::sync::Arc;

use super::{AudioStreamId, AudioStreamShared};

/// Controls an audio stream.
///
/// When an [`AudioStreamHandle`] is dropped, the corresponding
/// audio stream will be removed.
pub struct AudioStreamHandle {
	pub(crate) id: AudioStreamId,
	pub(crate) shared: Arc<AudioStreamShared>,
}

impl AudioStreamHandle {
	/// Returns the unique identifier for the audio stream.
	pub fn id(&self) -> AudioStreamId {
		self.id
	}
}

impl Drop for AudioStreamHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
