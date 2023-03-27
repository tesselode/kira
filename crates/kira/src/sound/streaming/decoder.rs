pub(crate) mod symphonia;

use crate::dsp::Frame;

pub(crate) trait Decoder: Send {
	type Error;

	fn sample_rate(&self) -> u32;

	fn decode(&mut self) -> Result<DecodeResponse, Self::Error>;

	fn seek(&mut self, index: u64) -> Result<SeekedToIndex, Self::Error>;
}

pub(crate) enum DecodeResponse {
	DecodedFrames(Vec<Frame>),
	ReachedEndOfAudio,
}

type SeekedToIndex = u64;
