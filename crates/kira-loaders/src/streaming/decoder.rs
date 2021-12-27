pub(crate) mod symphonia;

use std::collections::VecDeque;

use kira::dsp::Frame;

pub(crate) trait Decoder: Send {
	type Error;

	fn sample_rate(&self) -> u32;

	fn decode(&mut self, frames: &mut VecDeque<Frame>) -> Result<bool, Self::Error>;

	fn seek(&mut self, index: u64) -> Result<u64, Self::Error>;
}
