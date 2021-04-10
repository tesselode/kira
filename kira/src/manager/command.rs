use crate::{
	metronome::Metronome, sequence::instance::SequenceInstance, sound::instance::Instance,
};

pub(crate) enum Command {
	StartInstance { instance: Instance },
	StartSequenceInstance(SequenceInstance),
	AddMetronome(Metronome),
}
