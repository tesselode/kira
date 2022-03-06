pub mod producer;

use atomic_arena::Key;

use crate::{
	clock::{Clock, ClockId},
	sound::Sound,
	spatial::scene::{SpatialScene, SpatialSceneId},
	track::{SubTrackId, Track, TrackId},
	tween::Tween,
	ClockSpeed, Volume,
};

pub(crate) enum SoundCommand {
	Add(Key, Box<dyn Sound>),
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, Track),
	SetTrackVolume(TrackId, Volume, Tween),
	SetTrackRoutes {
		from: TrackId,
		to: TrackId,
		volume: Volume,
		tween: Tween,
	},
}

pub(crate) enum ClockCommand {
	Add(ClockId, Clock),
	SetSpeed(ClockId, ClockSpeed, Tween),
	Start(ClockId),
	Pause(ClockId),
	Stop(ClockId),
}

pub(crate) enum SpatialSceneCommand {
	Add(SpatialSceneId, SpatialScene),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Mixer(MixerCommand),
	Clock(ClockCommand),
	SpatialScene(SpatialSceneCommand),
	Pause(Tween),
	Resume(Tween),
}
