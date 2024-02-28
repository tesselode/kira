pub mod producer;

use atomic_arena::Key;

use crate::{
	clock::{Clock, ClockId},
	modulator::{Modulator, ModulatorId},
	sound::wrapper::SoundWrapper,
	spatial::{
		emitter::{Emitter, EmitterId},
		listener::{Listener, ListenerId},
		scene::{SpatialScene, SpatialSceneId},
	},
	track::{SubTrackId, Track},
	tween::Tween,
};

pub(crate) enum SpatialSceneCommand {
	Add(SpatialSceneId, SpatialScene),
	AddEmitter(EmitterId, Emitter),
	AddListener(ListenerId, Listener),
}

pub(crate) enum ModulatorCommand {
	Add(ModulatorId, Box<dyn Modulator>),
}

pub(crate) enum Command {
	AddSound(Key, SoundWrapper),
	AddSubTrack(SubTrackId, Track),
	AddClock(ClockId, Clock),
	SpatialScene(SpatialSceneCommand),
	Modulator(ModulatorCommand),
	Pause(Tween),
	Resume(Tween),
}
