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
	SetInstanceVolume(InstanceId, f32, Option<Tween<f32>>),
	SetInstancePitch(InstanceId, f32, Option<Tween<f32>>),
	PauseInstance(InstanceId, Option<Tween<f32>>),
	ResumeInstance(InstanceId, Option<Tween<f32>>),
	StopInstance(InstanceId, Option<Tween<f32>>),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
	StartSequence(SequenceId, Sequence),
}
