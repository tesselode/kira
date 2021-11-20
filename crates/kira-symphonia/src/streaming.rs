mod data;
mod handle;
mod sound;

pub use data::*;
pub use handle::*;
use kira::{tween::Tween, value::Value};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Command {
	SetVolume(Value),
	SetPlaybackRate(Value),
	SetPanning(Value),
	Pause(Tween),
	Resume(Tween),
	Stop(Tween),
	SeekBy(f64),
	SeekTo(f64),
}
