pub mod producer;

use crate::{
	parameter::{tween::Tween, Parameter, ParameterId},
	sound::{
		instance::{Instance, InstanceId},
		Sound, SoundId,
	},
	track::{SubTrackId, Track},
	value::Value,
};

pub(crate) enum SoundCommand {
	Add(SoundId, Sound),
}

pub(crate) enum InstanceCommand {
	Add {
		id: InstanceId,
		instance: Instance,
		command_sent_time: u64,
	},
	SetVolume(InstanceId, Value),
	SetPlaybackRate(InstanceId, Value),
	SetPanning(InstanceId, Value),
	Pause {
		id: InstanceId,
		tween: Tween,
		command_sent_time: u64,
	},
	Resume {
		id: InstanceId,
		tween: Tween,
		command_sent_time: u64,
	},
	Stop {
		id: InstanceId,
		tween: Tween,
		command_sent_time: u64,
	},
}

pub(crate) enum ParameterCommand {
	Add(ParameterId, Parameter),
	Set {
		id: ParameterId,
		target: f64,
		tween: Tween,
		command_sent_time: u64,
	},
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, Track),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Instance(InstanceCommand),
	Parameter(ParameterCommand),
	Mixer(MixerCommand),
}
