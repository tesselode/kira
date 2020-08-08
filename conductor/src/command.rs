use crate::{
	instance::{InstanceId, InstanceSettings},
	metronome::MetronomeId,
	sequence::{Sequence, SequenceId},
	sound::SoundId,
};

#[derive(Debug, Clone)]
pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
	StartSequence(SequenceId, Sequence),
}
