//! Playable chunks of audio that are loaded into memory all at once.

mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use crate::{parameter::Value, tween::Tween, PlaybackRate, Volume};

use super::Region;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
	SetVolume(Value<Volume>, Tween),
	SetPlaybackRate(Value<PlaybackRate>, Tween),
	SetPanning(Value<f64>, Tween),
	SetPlaybackRegion(Region),
	SetLoopRegion(Option<Region>),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
