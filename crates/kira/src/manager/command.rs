pub mod producer;

use atomic_arena::Key;

use crate::{
	clock::{Clock, ClockId},
	sound::Sound,
	track::{SubTrackId, Track, TrackId},
	tween::Tween,
};

pub(crate) enum SoundCommand {
	Add(Key, Box<dyn Sound>),
}

pub(crate) enum MixerCommand {
	AddSubTrack(SubTrackId, Track),
	SetTrackVolume(TrackId, f64),
	SetTrackPanning(TrackId, f64),
}

pub(crate) enum ClockCommand {
	Add(ClockId, Clock),
	SetInterval(ClockId, f64, Tween),
	Start(ClockId),
	Pause(ClockId),
	Stop(ClockId),
}

pub(crate) enum Command {
	Sound(SoundCommand),
	Mixer(MixerCommand),
	Clock(ClockCommand),
	Pause(Tween),
	Resume(Tween),
}
