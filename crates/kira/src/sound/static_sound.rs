//! Playable chunks of audio that are loaded into memory all at once.

mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;
pub use sound::PlaybackState;

use crate::{tween::Tween, value::Value};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
	SetVolume(Value),
	SetPlaybackRate(Value),
	SetPanning(Value),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
