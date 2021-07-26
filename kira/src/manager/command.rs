use crate::sound::{Sound, SoundId};

pub(crate) enum SoundCommand {
	Add(SoundId, Sound),
}

pub(crate) enum Command {
	Sound(SoundCommand),
}
