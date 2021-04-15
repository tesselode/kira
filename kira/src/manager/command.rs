use basedrop::{Owned, Shared};

use crate::{
	metronome::Metronome,
	mixer::track::Track,
	parameter::Parameter,
	sequence::instance::SequenceInstance,
	sound::{instance::Instance, Sound},
};

pub(crate) enum Command {
	AddSound(Shared<Sound>),
	StartInstance { instance: Instance },
	StartSequenceInstance(SequenceInstance),
	AddMetronome(Metronome),
	AddParameter(Parameter),
	AddSubTrack(Owned<Track>),
}
