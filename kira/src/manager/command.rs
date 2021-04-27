use std::sync::Arc;

use basedrop::Owned;

use crate::{
	metronome::Metronome,
	mixer::track::Track,
	parameter::Parameter,
	sequence::instance::SequenceInstance,
	sound::{instance::Instance, Sound},
};

pub(crate) enum Command {
	AddSound(Owned<Arc<Sound>>),
	StartInstance(Instance),
	StartSequenceInstance(Owned<SequenceInstance>),
	AddMetronome(Owned<Metronome>),
	AddParameter(Owned<Parameter>),
	AddSubTrack(Owned<Track>),
}
