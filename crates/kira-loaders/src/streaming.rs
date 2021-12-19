mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use kira::tween::Tween;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Command {
	SetVolume(f64, Tween),
	SetPlaybackRate(f64, Tween),
	SetPanning(f64, Tween),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
