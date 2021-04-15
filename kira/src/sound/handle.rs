use basedrop::Shared;

use super::Sound;

#[derive(Clone)]
pub struct SoundHandle {
	sound: Shared<Sound>,
}

impl SoundHandle {
	pub(crate) fn new(sound: Shared<Sound>) -> Self {
		Self { sound }
	}

	pub(crate) fn sound(&self) -> &Shared<Sound> {
		&self.sound
	}
}
