use crate::{
	instance::{InstanceId, InstanceSettings},
	metronome::MetronomeId,
	sequence::{Sequence, SequenceId},
	sound::SoundId,
};

#[derive(Debug, Clone)]
pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	PauseInstance(InstanceId, Option<f32>),
	ResumeInstance(InstanceId, Option<f32>),
	StopInstance(InstanceId, Option<f32>),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
	StartSequence(SequenceId, Sequence),
}
