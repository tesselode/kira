use crate::{
	instance::{InstanceId, InstanceSettings},
	mixer::effect::Effect,
	mixer::effect::EffectId,
	mixer::{effect::EffectSettings, SubTrackId, TrackIndex, TrackSettings},
	parameter::{ParameterId, Tween},
	sequence::{Sequence, SequenceId},
	sound::{Sound, SoundId},
	tempo::Tempo,
	value::Value,
};

#[derive(Debug)]
pub(crate) enum SoundCommand {
	LoadSound(SoundId, Sound),
	UnloadSound(SoundId),
}

#[derive(Debug, Clone)]
pub(crate) enum InstanceCommand {
	PlaySound(InstanceId, SoundId, Option<SequenceId>, InstanceSettings),
	SetInstanceVolume(InstanceId, Value<f64>),
	SetInstancePitch(InstanceId, Value<f64>),
	SetInstancePanning(InstanceId, Value<f64>),
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

#[derive(Debug, Clone)]
pub(crate) enum MetronomeCommand {
	SetMetronomeTempo(Value<Tempo>),
	StartMetronome,
	PauseMetronome,
	StopMetronome,
}

#[derive(Debug, Clone)]
pub(crate) enum SequenceCommand<CustomEvent: Copy> {
	StartSequence(SequenceId, Sequence<CustomEvent>),
	MuteSequence(SequenceId),
	UnmuteSequence(SequenceId),
	PauseSequence(SequenceId),
	ResumeSequence(SequenceId),
	StopSequence(SequenceId),
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, TrackSettings),
	RemoveSubTrack(SubTrackId),
	AddEffect(TrackIndex, EffectId, Box<dyn Effect>, EffectSettings),
	RemoveEffect(EffectId),
}

#[derive(Debug, Clone)]
pub(crate) enum ParameterCommand {
	AddParameter(ParameterId, f64),
	RemoveParameter(ParameterId),
	SetParameter(ParameterId, f64, Option<Tween>),
}

pub(crate) enum Command<CustomEvent: Copy> {
	Sound(SoundCommand),
	Instance(InstanceCommand),
	Metronome(MetronomeCommand),
	Sequence(SequenceCommand<CustomEvent>),
	Mixer(MixerCommand),
	Parameter(ParameterCommand),
	EmitCustomEvent(CustomEvent),
}

impl<CustomEvent: Copy> From<SoundCommand> for Command<CustomEvent> {
	fn from(command: SoundCommand) -> Self {
		Self::Sound(command)
	}
}

impl<CustomEvent: Copy> From<InstanceCommand> for Command<CustomEvent> {
	fn from(command: InstanceCommand) -> Self {
		Self::Instance(command)
	}
}

impl<CustomEvent: Copy> From<MetronomeCommand> for Command<CustomEvent> {
	fn from(command: MetronomeCommand) -> Self {
		Self::Metronome(command)
	}
}

impl<CustomEvent: Copy> From<SequenceCommand<CustomEvent>> for Command<CustomEvent> {
	fn from(command: SequenceCommand<CustomEvent>) -> Self {
		Self::Sequence(command)
	}
}

impl<CustomEvent: Copy> From<MixerCommand> for Command<CustomEvent> {
	fn from(command: MixerCommand) -> Self {
		Self::Mixer(command)
	}
}

impl<CustomEvent: Copy> From<ParameterCommand> for Command<CustomEvent> {
	fn from(command: ParameterCommand) -> Self {
		Self::Parameter(command)
	}
}
