//! Decodes data gradually from an audio file.

mod data;
pub(crate) mod decoder;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use crate::{tween::Tween, PlaybackRate, Volume};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Command {
	SetVolume(Volume, Tween),
	SetPlaybackRate(PlaybackRate, Tween),
	SetPanning(f64, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
