use crate::{
	id::{InstanceId, MetronomeId, SequenceId, SoundId},
	manager::InstanceSettings,
	sequence::Sequence,
};

#[derive(Debug, Clone)]
pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
	StartSequence(SequenceId, Sequence),
}
