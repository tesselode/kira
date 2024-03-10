pub mod producer;

use atomic_arena::Key;

use crate::{
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

pub(crate) enum Command {
	AddSound(Key, SoundWrapper),
	AddSubTrack(SubTrackId, Track),
	SpatialScene(SpatialSceneCommand),
	Pause(Tween),
	Resume(Tween),
}
