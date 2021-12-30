use std::collections::VecDeque;

use kira::{
	dsp::Frame,
	sound::{static_sound::PlaybackState, Sound},
};

use crate::{
	decoder::Decoder, streaming::sound::decode_scheduler::NextStep, StreamingSoundData,
	StreamingSoundSettings,
};

const MOCK_DECODER_SAMPLE_RATE: u32 = 1;
const MOCK_DECODER_PACKET_SIZE: usize = 3;

struct MockDecoder {
	frames: Vec<Frame>,
	current_frame_index: usize,
}

impl MockDecoder {
	fn new(frames: Vec<Frame>) -> Self {
		Self {
			frames,
			current_frame_index: 0,
		}
	}
}

impl Decoder for MockDecoder {
	type Error = ();

	fn sample_rate(&self) -> u32 {
		MOCK_DECODER_SAMPLE_RATE
	}

	fn decode(&mut self, frames: &mut VecDeque<Frame>) -> Result<bool, Self::Error> {
		if self.current_frame_index >= self.frames.len() {
			return Ok(true);
		}
		for _ in 0..MOCK_DECODER_PACKET_SIZE {
			frames.push_back(self.frames[self.current_frame_index]);
			self.current_frame_index += 1;
			if self.current_frame_index >= self.frames.len() {
				break;
			}
		}
		Ok(false)
	}

	fn seek(&mut self, index: u64) -> Result<u64, Self::Error> {
		// seek to the beginning of the "packet" to simulate
		// seeking behavior with real decoders
		let index = (index as f64 / MOCK_DECODER_PACKET_SIZE as f64) as u64
			* MOCK_DECODER_PACKET_SIZE as u64;
		Ok(index)
	}
}

/// Tests that a `StreamingSound` will play all of its samples before finishing.
#[test]
fn plays_all_samples() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(1..=10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::default(),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while !matches!(scheduler.run().unwrap(), NextStep::End) {}

	assert!(!sound.finished());
	assert_eq!(sound.state, PlaybackState::Playing);

	for i in 1..=10 {
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
		assert!(!sound.finished());
		assert_eq!(sound.state, PlaybackState::Playing);
	}

	assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
	assert!(sound.finished());
	assert_eq!(sound.state, PlaybackState::Stopped);
}
