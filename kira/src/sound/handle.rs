use std::sync::{atomic::Ordering, Arc};

use super::{SoundId, SoundShared};

pub struct SoundHandle {
	pub(crate) id: SoundId,
	pub(crate) shared: Arc<SoundShared>,
}

impl SoundHandle {
	pub fn id(&self) -> SoundId {
		self.id
	}
}

impl Drop for SoundHandle {
	fn drop(&mut self) {
		self.shared.removed.store(true, Ordering::SeqCst);
	}
}
