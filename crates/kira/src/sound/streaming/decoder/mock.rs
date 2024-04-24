use crate::dsp::Frame;

use super::Decoder;

const MOCK_DECODER_SAMPLE_RATE: u32 = 1;
const MOCK_DECODER_PACKET_SIZE: usize = 3;

pub(crate) struct MockDecoder {
	frames: Vec<Frame>,
	current_frame_index: usize,
}

impl MockDecoder {
	#[must_use]
	pub(crate) fn new(frames: Vec<Frame>) -> Self {
		Self {
			frames,
			current_frame_index: 0,
		}
	}
}

impl Decoder for MockDecoder {
	type Error = MockDecoderError;

	fn sample_rate(&self) -> u32 {
		MOCK_DECODER_SAMPLE_RATE
	}

	fn num_frames(&self) -> usize {
		self.frames.len()
	}

	fn decode(&mut self) -> Result<Vec<Frame>, Self::Error> {
		let mut frames = vec![];
		for _ in 0..MOCK_DECODER_PACKET_SIZE {
			let frame = self.frames[self.current_frame_index];
			if frame.left.is_nan() || frame.right.is_nan() {
				return Err(MockDecoderError);
			}
			frames.push(frame);
			self.current_frame_index += 1;
			if self.current_frame_index >= self.frames.len() {
				break;
			}
		}
		Ok(frames)
	}

	fn seek(&mut self, index: usize) -> Result<usize, Self::Error> {
		// seek to the beginning of the "packet" to simulate
		// seeking behavior with real decoders
		let index =
			(index as f64 / MOCK_DECODER_PACKET_SIZE as f64) as usize * MOCK_DECODER_PACKET_SIZE;
		self.current_frame_index = index;
		Ok(index)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MockDecoderError;
