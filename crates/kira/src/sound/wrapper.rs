use super::{CommonSoundSettings, Sound};

pub(crate) struct SoundWrapper {
	pub(crate) sound: Box<dyn Sound>,
}

impl SoundWrapper {
	pub fn new(sound: Box<dyn Sound>, settings: CommonSoundSettings) -> Self {
		Self { sound }
	}
}
