mod data;
mod settings;
mod sound;

pub use data::*;
pub use settings::*;
pub use sound::*;

use std::collections::VecDeque;

use kira::dsp::Frame;

pub trait Decoder: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn decode(&mut self) -> Option<VecDeque<Frame>>;

	fn reset(&mut self);
}
