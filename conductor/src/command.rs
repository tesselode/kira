use crate::{
	instance::{InstanceId, InstanceSettings},
	sequence::{Sequence, SequenceId},
	sound::{Sound, SoundId},
	tween::Tween,
};

pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
	UnloadSound(SoundId),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum InstanceCommand<Id> {
	PlaySound(SoundId, Id, InstanceSettings),
	SetInstanceVolume(Id, f32, Option<Tween>),
	SetInstancePitch(Id, f32, Option<Tween>),
	PauseInstance(Id, Option<Tween>),
	ResumeInstance(Id, Option<Tween>),
	StopInstance(Id, Option<Tween>),
	PauseInstancesOfSound(SoundId, Option<Tween>),
	ResumeInstancesOfSound(SoundId, Option<Tween>),
	StopInstancesOfSound(SoundId, Option<Tween>),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum MetronomeCommand {
	StartMetronome,
	PauseMetronome,
	StopMetronome,
}

pub(crate) enum SequenceCommand {
	StartSequence(SequenceId, Sequence),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Instance(InstanceCommand<InstanceId>),
	Metronome(MetronomeCommand),
	Sequence(SequenceCommand),
}
