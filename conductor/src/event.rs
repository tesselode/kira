use crate::sound_bank::SoundId;

pub enum Command {
	PlaySound(SoundId),
}
