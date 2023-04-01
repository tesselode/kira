//! Playable chunks of audio that are loaded into memory all at once.

mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use crate::{parameter::Value, tween::Tween, PlaybackRate, Volume};

use super::{LoopRegion, PlaybackRegion};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
	SetVolume(Value<Volume>, Tween),
	SetPlaybackRate(Value<PlaybackRate>, Tween),
	SetPanning(Value<f64>, Tween),
	SetPlaybackRegion(PlaybackRegion),
	SetLoopRegion(Option<LoopRegion>),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
