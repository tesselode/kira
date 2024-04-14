pub mod producer;

use atomic_arena::Key;

use crate::{
	clock::{Clock, ClockId},
	modulator::{Modulator, ModulatorId},
	sound::Sound,
	spatial::{
		emitter::{Emitter, EmitterId},
		listener::{Listener, ListenerId},
		scene::{SpatialScene, SpatialSceneId},
	},
	track::{SubTrackId, Track},
	tween::Tween,
};

pub(crate) enum SoundCommand {
	Add(Key, Box<dyn Sound>),
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, Track),
}

pub(crate) enum ClockCommand {
	Add(ClockId, Clock),
}

pub(crate) enum SpatialSceneCommand {
	Add(SpatialSceneId, SpatialScene),
	AddEmitter(EmitterId, Emitter),
	AddListener(ListenerId, Listener),
}

pub(crate) enum ModulatorCommand {
	Add(ModulatorId, Box<dyn Modulator>),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Mixer(MixerCommand),
	Clock(ClockCommand),
	SpatialScene(SpatialSceneCommand),
	Modulator(ModulatorCommand),
	Pause(Tween),
	Resume(Tween),
}
