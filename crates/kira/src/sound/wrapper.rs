use super::Sound;

pub(crate) struct SoundWrapper {
	pub(crate) sound: Box<dyn Sound>,
}
