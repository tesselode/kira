use crate::{
	instance::{InstanceId, InstanceSettings},
	sound::{Sound, SoundId},
	tween::Tween,
};

pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
}

pub(crate) enum InstanceCommand {
	PlaySound(SoundId, InstanceId, InstanceSettings),
	SetInstanceVolume(InstanceId, f32, Option<Tween>),
	SetInstancePitch(InstanceId, f32, Option<Tween>),
	PauseInstance(InstanceId, Option<Tween>),
	ResumeInstance(InstanceId, Option<Tween>),
	StopInstance(InstanceId, Option<Tween>),
	PauseInstancesOfSound(SoundId, Option<Tween>),
	ResumeInstancesOfSound(SoundId, Option<Tween>),
	StopInstancesOfSound(SoundId, Option<Tween>),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Instance(InstanceCommand),
}
