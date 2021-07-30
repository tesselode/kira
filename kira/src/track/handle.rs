use std::sync::Arc;

use super::{TrackId, TrackShared};

pub struct TrackHandle {
	pub(crate) id: TrackId,
	pub(crate) shared: Arc<TrackShared>,
}

impl TrackHandle {
	pub fn id(&self) -> TrackId {
		self.id
	}
}

impl Drop for TrackHandle {
	fn drop(&mut self) {
		self.shared.mark_for_removal();
	}
}
