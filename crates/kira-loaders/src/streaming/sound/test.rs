use std::{collections::VecDeque, time::Duration};

use kira::{
	dsp::Frame,
	sound::{static_sound::PlaybackState, Sound},
	tween::Tween,
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

/// Tests that a `StreamingSound` will pause playback while waiting
/// for samples from the decoder.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_samples() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(1..=10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::default(),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::ZERO.panned(0.5));
		assert_eq!(sound.position(), 0.0);
	}

	for _ in 0..4 {
		scheduler.run().unwrap();
	}

	for i in 1..=3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
		assert_eq!(sound.position(), (i - 1) as f64);
	}

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::ZERO.panned(0.5));
		assert_eq!(sound.position(), 2.0);
	}
}

/// Tests that a `StreamingSound` correctly reports its playback state
/// to be queried by StreamingSoundHandle::state.
#[test]
fn reports_playback_state() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.0); 2])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while !matches!(scheduler.run().unwrap(), NextStep::End) {}

	assert_eq!(handle.state(), PlaybackState::Playing);

	for _ in 0..2 {
		sound.process(1.0);
		sound.on_start_processing();
		assert_eq!(handle.state(), PlaybackState::Playing);
	}

	sound.process(1.0);
	sound.on_start_processing();
	assert_eq!(handle.state(), PlaybackState::Stopped);
}

/// Tests that a `StreamingSound` correctly reports its playback state
/// to be queried by StreamingSoundHandle::state.
#[test]
#[allow(clippy::float_cmp)]
fn reports_playback_position() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.0); 2])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while !matches!(scheduler.run().unwrap(), NextStep::End) {}

	assert_eq!(handle.position(), 0.0);

	for i in 1..=2 {
		sound.process(1.0);
		sound.on_start_processing();
		assert_eq!(handle.position(), (i - 1) as f64);
	}

	sound.process(1.0);
	sound.on_start_processing();
	// TODO: figure out whether the final position ought to be
	// 1.0 or 2.0
	assert_eq!(handle.position(), 1.0);
}

/// Tests that a `StreamingSound` fades out fully before pausing
/// and fades back in when resuming.
#[test]
#[allow(clippy::float_cmp)]
fn pauses_and_resumes_with_fades() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 100])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while !matches!(scheduler.run().unwrap(), NextStep::End) {}

	sound.process(1.0);
	assert_eq!(sound.position(), 0.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.pause(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.75).panned(0.5));
	assert_eq!(sound.position(), 1.0);
	assert_eq!(sound.state, PlaybackState::Pausing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
	assert_eq!(sound.position(), 2.0);
	assert_eq!(sound.state, PlaybackState::Pausing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.25).panned(0.5));
	assert_eq!(sound.position(), 3.0);
	assert_eq!(sound.state, PlaybackState::Pausing);

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
		assert_eq!(sound.position(), 4.0);
		assert_eq!(sound.state, PlaybackState::Paused);
	}

	handle
		.resume(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.25).panned(0.5));
	assert_eq!(sound.position(), 5.0);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
	assert_eq!(sound.position(), 6.0);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.75).panned(0.5));
	assert_eq!(sound.position(), 7.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	for i in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(1.0).panned(0.5));
		assert_eq!(sound.position(), i as f64 + 8.0);
		assert_eq!(sound.state, PlaybackState::Playing);
	}
}
