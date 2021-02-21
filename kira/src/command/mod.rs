pub mod producer;

use basedrop::Owned;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	audio_stream::{AudioStream, AudioStreamId},
	group::{Group, GroupId},
	instance::{
		Instance, InstanceId, PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings,
	},
	metronome::{Metronome, MetronomeId},
	mixer::{
		effect::{Effect, EffectId, EffectSettings},
		SendTrackId, SubTrackId, Track, TrackIndex,
	},
	parameter::{tween::Tween, ParameterId},
	playable::PlayableId,
	sequence::{SequenceInstance, SequenceInstanceId},
	sound::{Sound, SoundId},
	tempo::Tempo,
	value::Value,
};

#[derive(Clone)]
pub(crate) enum ResourceCommand {
	AddSound(Owned<Sound>),
	RemoveSound(SoundId),
	AddArrangement(Owned<Arrangement>),
	RemoveArrangement(ArrangementId),
}

#[derive(Debug, Clone)]
pub(crate) enum InstanceCommand {
	Play(InstanceId, Instance),
	SetInstanceVolume(InstanceId, Value<f64>),
	SetInstancePitch(InstanceId, Value<f64>),
	SetInstancePanning(InstanceId, Value<f64>),
	SeekInstance(InstanceId, f64),
	SeekInstanceTo(InstanceId, f64),
	PauseInstance(InstanceId, PauseInstanceSettings),
	ResumeInstance(InstanceId, ResumeInstanceSettings),
	StopInstance(InstanceId, StopInstanceSettings),
	PauseInstancesOf(PlayableId, PauseInstanceSettings),
	ResumeInstancesOf(PlayableId, ResumeInstanceSettings),
	StopInstancesOf(PlayableId, StopInstanceSettings),
	PauseInstancesOfSequence(SequenceInstanceId, PauseInstanceSettings),
	ResumeInstancesOfSequence(SequenceInstanceId, ResumeInstanceSettings),
	StopInstancesOfSequence(SequenceInstanceId, StopInstanceSettings),
	PauseGroup(GroupId, PauseInstanceSettings),
	ResumeGroup(GroupId, ResumeInstanceSettings),
	StopGroup(GroupId, StopInstanceSettings),
}

pub(crate) enum MetronomeCommand {
	AddMetronome(MetronomeId, Owned<Metronome>),
	RemoveMetronome(MetronomeId),
	SetMetronomeTempo(MetronomeId, Value<Tempo>),
	StartMetronome(MetronomeId),
	PauseMetronome(MetronomeId),
	StopMetronome(MetronomeId),
}

pub(crate) enum SequenceCommand {
	StartSequenceInstance(SequenceInstanceId, Owned<SequenceInstance>),
	MuteSequenceInstance(SequenceInstanceId),
	UnmuteSequenceInstance(SequenceInstanceId),
	PauseSequenceInstance(SequenceInstanceId),
	ResumeSequenceInstance(SequenceInstanceId),
	StopSequenceInstance(SequenceInstanceId),
	PauseGroup(GroupId),
	ResumeGroup(GroupId),
	StopGroup(GroupId),
}

pub(crate) enum MixerCommand {
	AddTrack(Owned<Track>),
	RemoveSubTrack(SubTrackId),
	RemoveSendTrack(SendTrackId),
	AddEffect(TrackIndex, Owned<Box<dyn Effect>>, EffectSettings),
	SetEffectEnabled(TrackIndex, EffectId, bool),
	RemoveEffect(TrackIndex, EffectId),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum ParameterCommand {
	AddParameter(ParameterId, f64),
	RemoveParameter(ParameterId),
	SetParameter(ParameterId, f64, Option<Tween>),
}

#[derive(Clone)]
pub(crate) enum GroupCommand {
	AddGroup(GroupId, Owned<Group>),
	RemoveGroup(GroupId),
}

pub(crate) enum StreamCommand {
	AddStream(AudioStreamId, TrackIndex, Owned<Box<dyn AudioStream>>),
	RemoveStream(AudioStreamId),
}

pub(crate) enum Command {
	Resource(ResourceCommand),
	Instance(InstanceCommand),
	Metronome(MetronomeCommand),
	Sequence(SequenceCommand),
	Mixer(MixerCommand),
	Parameter(ParameterCommand),
	Group(GroupCommand),
	Stream(StreamCommand),
}

impl From<ResourceCommand> for Command {
	fn from(command: ResourceCommand) -> Self {
		Self::Resource(command)
	}
}

impl From<InstanceCommand> for Command {
	fn from(command: InstanceCommand) -> Self {
		Self::Instance(command)
	}
}

impl From<MetronomeCommand> for Command {
	fn from(command: MetronomeCommand) -> Self {
		Self::Metronome(command)
	}
}

impl From<SequenceCommand> for Command {
	fn from(command: SequenceCommand) -> Self {
		Self::Sequence(command)
	}
}

impl From<MixerCommand> for Command {
	fn from(command: MixerCommand) -> Self {
		Self::Mixer(command)
	}
}

impl From<ParameterCommand> for Command {
	fn from(command: ParameterCommand) -> Self {
		Self::Parameter(command)
	}
}

impl From<GroupCommand> for Command {
	fn from(command: GroupCommand) -> Self {
		Self::Group(command)
	}
}

impl From<StreamCommand> for Command {
	fn from(command: StreamCommand) -> Self {
		Self::Stream(command)
	}
}
