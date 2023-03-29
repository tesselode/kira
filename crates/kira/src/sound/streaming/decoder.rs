pub(crate) mod symphonia;

use crate::dsp::Frame;

pub(crate) trait Decoder: Send {
	type Error;

	fn sample_rate(&self) -> u32;

	fn num_frames(&self) -> usize;

	fn decode(&mut self) -> Result<Vec<Frame>, Self::Error>;

	fn seek(&mut self, index: usize) -> Result<SeekedToIndex, Self::Error>;
}

type SeekedToIndex = usize;
