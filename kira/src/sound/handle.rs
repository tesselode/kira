use std::sync::Arc;

use super::Sound;

#[derive(Clone)]
pub struct SoundHandle {
	sound: Arc<Sound>,
}

impl SoundHandle {
	pub(crate) fn new(sound: Arc<Sound>) -> Self {
		Self { sound }
	}

	pub(crate) fn sound(&self) -> &Arc<Sound> {
		&self.sound
	}
}
