use crate::{
	metronome::Metronome, mixer::track::SubTrack, parameter::Parameter,
	sequence::instance::SequenceInstance, sound::instance::Instance,
};

pub(crate) enum Command {
	StartInstance { instance: Instance },
	StartSequenceInstance(SequenceInstance),
	AddMetronome(Metronome),
	AddParameter(Parameter),
	AddSubTrack(SubTrack),
}
