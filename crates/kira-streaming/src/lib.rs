mod data;
mod handle;
mod settings;
mod sound;

pub use data::*;
pub use handle::*;
pub use settings::*;

use std::collections::VecDeque;

use kira::{dsp::Frame, tween::Tween, value::Value};

pub trait Decoder: Send + Sync {
	type Error: Send + Sync;

	fn sample_rate(&mut self) -> u32;

	fn decode(&mut self) -> Result<Option<VecDeque<Frame>>, Self::Error>;

	fn reset(&mut self) -> Result<(), Self::Error>;
}

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
