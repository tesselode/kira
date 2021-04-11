use crate::{
	metronome::Metronome, parameter::Parameter, sequence::instance::SequenceInstance,
	sound::instance::Instance,
};

pub(crate) enum Command {
	StartInstance { instance: Instance },
	StartSequenceInstance(SequenceInstance),
	AddMetronome(Metronome),
	AddParameter(Parameter),
}
