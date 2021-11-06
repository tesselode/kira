mod data;
mod sound;

pub use data::*;
pub use sound::*;

use std::collections::VecDeque;

use crate::dsp::Frame;

pub trait Decoder: Send + Sync {
	fn sample_rate(&mut self) -> u32;

	fn decode(&mut self) -> Option<VecDeque<Frame>>;
}
