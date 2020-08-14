use crate::{
	instance::{InstanceId, InstanceSettings},
	sound::{Sound, SoundId},
	tween::Tween,
};

pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
	UnloadSound(SoundId),
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
pub(crate) enum MetronomeCommand {
	StartMetronome,
	PauseMetronome,
	StopMetronome,
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Instance(InstanceCommand<InstanceId>),
	Metronome(MetronomeCommand),
}
