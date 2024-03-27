pub mod producer;

use atomic_arena::Key;
use glam::{Quat, Vec3};

use crate::{
	clock::{ClockId, ClockSpeed},
	sound::Sound,
	spatial::{
		emitter::{Emitter, EmitterId},
		listener::{Listener, ListenerId},
		scene::{SpatialScene, SpatialSceneId},
	},
	track::{SubTrackId, Track, TrackId},
	tween::{Tween, Value},
	Volume,
};

use super::backend::resources::clocks::buffered::BufferedClock;

pub(crate) enum SoundCommand {
	Add(Key, Box<dyn Sound>),
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, Track),
	SetTrackVolume(TrackId, Value<Volume>, Tween),
	SetTrackRoutes {
		from: TrackId,
		to: TrackId,
		volume: Value<Volume>,
		tween: Tween,
	},
}

pub(crate) enum ClockCommand {
	Add(ClockId, BufferedClock),
	SetSpeed(ClockId, Value<ClockSpeed>, Tween),
	Start(ClockId),
	Pause(ClockId),
	Stop(ClockId),
}

pub(crate) enum SpatialSceneCommand {
	Add(SpatialSceneId, SpatialScene),
	AddEmitter(EmitterId, Emitter),
	AddListener(ListenerId, Listener),
	SetListenerPosition(ListenerId, Value<Vec3>, Tween),
	SetListenerOrientation(ListenerId, Value<Quat>, Tween),
	SetEmitterPosition(EmitterId, Value<Vec3>, Tween),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Mixer(MixerCommand),
	Clock(ClockCommand),
	SpatialScene(SpatialSceneCommand),
}
