pub mod producer;

use crate::sound::{
	instance::{Instance, InstanceId},
	Sound, SoundId,
};

pub(crate) enum SoundCommand {
	Add(SoundId, Sound),
}

pub(crate) enum InstanceCommand {
	Add(InstanceId, Instance),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Instance(InstanceCommand),
}
