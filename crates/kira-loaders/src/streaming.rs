mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use kira::{tween::Tween, PlaybackRate};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Command {
	SetVolume(f64, Tween),
	SetPlaybackRate(PlaybackRate, Tween),
	SetPanning(f64, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
