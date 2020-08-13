use crate::sound::{Sound, SoundId};

pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
}

pub(crate) enum Command {
	Sound(SoundCommand),
}
