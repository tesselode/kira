use crate::{
	instance::{InstanceId, InstanceSettings},
	parameter::ParameterId,
	sequence::{Sequence, SequenceId},
	sound::{Sound, SoundId},
	tempo::Tempo,
	track::effect::Effect,
	track::effect::EffectId,
	track::{id::SubTrackId, index::TrackIndex, EffectSettings, TrackSettings},
	tween::Tween,
	value::Value,
};

pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
	UnloadSound(SoundId),
}

#[derive(Debug, Clone)]
pub(crate) enum InstanceCommand {
	PlaySound(InstanceId, SoundId, Option<SequenceId>, InstanceSettings),
	SetInstanceVolume(InstanceId, f64, Option<Tween>),
	SetInstancePitch(InstanceId, f64, Option<Tween>),
	PauseInstance(InstanceId, Option<Tween>),
	ResumeInstance(InstanceId, Option<Tween>),
	StopInstance(InstanceId, Option<Tween>),
	PauseInstancesOfSound(SoundId, Option<Tween>),
	ResumeInstancesOfSound(SoundId, Option<Tween>),
	StopInstancesOfSound(SoundId, Option<Tween>),
	PauseInstancesOfSequence(SequenceId, Option<Tween>),
	ResumeInstancesOfSequence(SequenceId, Option<Tween>),
	StopInstancesOfSequence(SequenceId, Option<Tween>),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum MetronomeCommand {
	SetMetronomeTempo(Tempo),
	StartMetronome,
	PauseMetronome,
	StopMetronome,
}

pub(crate) enum SequenceCommand<CustomEvent> {
	StartSequence(SequenceId, Sequence<CustomEvent>),
	MuteSequence(SequenceId),
	UnmuteSequence(SequenceId),
	PauseSequence(SequenceId),
	ResumeSequence(SequenceId),
	StopSequence(SequenceId),
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, TrackSettings),
	AddEffect(TrackIndex, EffectId, Box<dyn Effect + Send>, EffectSettings),
}

pub(crate) enum ParameterCommand {
	AddParameter(ParameterId, f64),
	SetParameter(ParameterId, f64, Option<Tween>),
}

pub(crate) enum Command<CustomEvent> {
	Sound(SoundCommand),
	Instance(InstanceCommand),
	Metronome(MetronomeCommand),
	Sequence(SequenceCommand<CustomEvent>),
	Mixer(MixerCommand),
	Parameter(ParameterCommand),
	EmitCustomEvent(CustomEvent),
}
