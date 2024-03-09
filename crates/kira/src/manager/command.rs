pub mod producer;

use atomic_arena::Key;

use crate::{
	clock::ClockId,
	modulator::ModulatorId,
	sound::wrapper::SoundWrapper,
	spatial::{
		emitter::{Emitter, EmitterId},
		listener::{Listener, ListenerId},
		scene::{SpatialScene, SpatialSceneId},
	},
	track::{SubTrackId, Track},
};

use super::backend::resources::{
	clocks::buffered::BufferedClock, modulators::buffered::BufferedModulator,
};

pub(crate) enum SpatialSceneCommand {
	Add(SpatialSceneId, SpatialScene),
	AddEmitter(EmitterId, Emitter),
	AddListener(ListenerId, Listener),
}

pub(crate) enum ModulatorCommand {
	Add(ModulatorId, BufferedModulator),
}

pub(crate) enum Command {
	AddSound(Key, SoundWrapper),
	AddSubTrack(SubTrackId, Track),
	AddClock(ClockId, BufferedClock),
	SpatialScene(SpatialSceneCommand),
	Modulator(ModulatorCommand),
}
