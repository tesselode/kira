use crate::{manager::PlaySoundSettings, sound_bank::SoundId};

pub enum Command {
	PlaySound(SoundId, PlaySoundSettings),
}
