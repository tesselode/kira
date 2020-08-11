use crate::{
	instance::{InstanceId, InstanceSettings},
	metronome::MetronomeId,
	sequence::{Sequence, SequenceId},
	sound::SoundId,
	tween::Tween,
};

#[derive(Debug, Clone)]
pub enum Command {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	SetInstanceVolume(InstanceId, f32, Option<Tween>),
	SetInstancePitch(InstanceId, f32, Option<Tween>),
	PauseInstance(InstanceId, Option<f32>),
	ResumeInstance(InstanceId, Option<f32>),
	StopInstance(InstanceId, Option<f32>),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
	StartSequence(SequenceId, Sequence),
}
