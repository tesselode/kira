use std::{collections::VecDeque, time::Duration};

use kira::{
	dsp::Frame,
	sound::{static_sound::PlaybackState, Sound},
	tween::Tween,
	LoopBehavior, Volume,
};

use crate::{
	decoder::Decoder, streaming::sound::decode_scheduler::NextStep, StreamingSoundData,
	StreamingSoundSettings,
};

use super::StreamingSound;

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
		self.current_frame_index = index as usize;
		Ok(index)
	}
}

/// Tests that a `StreamingSound` will play all of its samples before finishing.
#[test]
fn plays_all_samples() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert!(!sound.finished());

	for i in 1..=3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
		assert!(!sound.finished());
	}

	// give some time for the resample buffer to empty. in the meantime we should
	// get silent output.
	for _ in 0..10 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
	}

	// the sound should be finished and stopped by now
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
	let (mut sound, handle, mut scheduler) = data.split().unwrap();

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::ZERO.panned(0.5));
		sound.on_start_processing();
		assert_eq!(handle.position(), 0.0);
	}

	for _ in 0..4 {
		scheduler.run().unwrap();
	}

	for i in 1..=3 {
		assert_eq!(handle.position(), (i - 1) as f64);
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
		sound.on_start_processing();
	}

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::ZERO.panned(0.5));
		sound.on_start_processing();
		assert_eq!(handle.position(), 2.0);
	}
}

/// Tests that a `StreamingSound` correctly reports its playback state
/// to be queried by StreamingSoundHandle::state.
#[test]
fn reports_playback_state() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.0); 10])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for _ in 0..20 {
		assert_eq!(handle.state(), sound.state);
		sound.process(1.0);
	}
}

/// Tests that a `StreamingSound` correctly reports its playback state
/// to be queried by StreamingSoundHandle::state.
#[test]
#[allow(clippy::float_cmp)]
fn reports_playback_position() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.0); 10])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for i in 0..20 {
		assert_eq!(handle.position(), i.clamp(0, 9) as f64);
		sound.process(1.0);
		sound.on_start_processing();
	}
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
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(1.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.pause(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();
	assert_eq!(sound.state, PlaybackState::Pausing);

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade out soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5),
		&mut sound,
	);
	assert_eq!(
		sound.process(1.0),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(
		sound.process(1.0),
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5)
	);
	sound.process(1.0);

	sound.on_start_processing();
	let position = handle.position();
	for _ in 0..10 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
		sound.on_start_processing();
		assert_eq!(handle.position(), position);
		assert_eq!(sound.state, PlaybackState::Paused);
	}

	handle
		.resume(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade back in soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5),
		&mut sound,
	);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(
		sound.process(1.0),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(
		sound.process(1.0),
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(sound.state, PlaybackState::Playing);

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(1.0).panned(0.5));
		assert_eq!(sound.state, PlaybackState::Playing);
	}
}

/// Tests that a `StreamingSound` stops and finishes after a fade-out.
#[test]
#[allow(clippy::float_cmp)]
fn stops_with_fade_out() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 100])),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(1.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.stop(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();
	assert_eq!(sound.state, PlaybackState::Stopping);

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade out soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5),
		&mut sound,
	);
	assert_eq!(
		sound.process(1.0),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(
		sound.process(1.0),
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5)
	);
	sound.process(1.0);

	sound.on_start_processing();
	let position = handle.position();
	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
		sound.on_start_processing();
		assert_eq!(handle.position(), position);
		assert_eq!(sound.state, PlaybackState::Stopped);
		assert!(sound.finished());
	}
}

/// Tests that a `StreamingSound` can be started partway through the sound.
#[test]
#[allow(clippy::float_cmp)]
fn start_position() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().start_position(3.0),
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(handle.position(), 3.0);
	assert_eq!(sound.process(1.0), Frame::from_mono(3.0).panned(0.5));
}

/// Tests that a `StreamingSound` properly obeys looping behavior when
/// playing forward.
#[test]
#[allow(clippy::float_cmp)]
fn loops_forward() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().loop_behavior(LoopBehavior {
			start_position: 3.0,
		}),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for i in 0..10 {
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
	}

	assert_eq!(sound.process(3.0), Frame::from_mono(3.0).panned(0.5));
	assert_eq!(sound.process(3.0), Frame::from_mono(6.0).panned(0.5));
	assert_eq!(sound.process(3.0), Frame::from_mono(9.0).panned(0.5));
	assert_eq!(sound.process(3.0), Frame::from_mono(5.0).panned(0.5));
}

/// Tests that the volume of a `StreamingSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn volume() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 10])),
		settings: StreamingSoundSettings::new().volume(0.5),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
}

/// Tests that the panning of a `StreamingSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn panning() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 10])),
		settings: StreamingSoundSettings::new().panning(0.0),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(sound.process(1.0), Frame::from_mono(1.0).panned(0.0));
}

/// Tests that the playback rate of a `StreamingSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn playback_rate() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().playback_rate(2.0),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
	assert_eq!(sound.process(1.0), Frame::from_mono(2.0).panned(0.5));
}

/// Tests that a `StreamingSound` outputs interpolated samples when
/// its playback position is between samples.
#[test]
fn interpolates_samples() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![
			Frame::from_mono(0.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(-10.0),
		])),
		settings: Default::default(),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(sound.process(0.5), Frame::from_mono(0.0).panned(0.5));
	// at sample 0.5, the output should be somewhere between 0 and 1.
	// i don't care what exactly, that's up the to the interpolation algorithm.
	let frame = sound.process(5.0);
	assert!(frame.left > 0.0 && frame.left < 1.0);
	// at sample 5.5, the output should be between 1 and -10.
	let frame = sound.process(1.0);
	assert!(frame.left < 0.0 && frame.left > -10.0);
}

/// Tests that a `StreamingSound` outputs interpolated samples correctly
/// when looping.
#[test]
fn interpolates_samples_when_looping() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![
			Frame::from_mono(10.0),
			Frame::from_mono(9.0),
		])),
		settings: StreamingSoundSettings::new().loop_behavior(LoopBehavior {
			start_position: 0.0,
		}),
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(1.5);
	// because we're looping back to the first sample, which is 10.0,
	// the interpolated sample should be be tween 9.0 and 10.0
	let frame = sound.process(1.0);
	assert!(frame.left > 9.0 && frame.left < 10.0);
}

/// Tests that a `StreamingSound` can seek to a position.
#[test]
fn seek_to() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new(),
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();

	handle.seek_to(15.0).unwrap();
	sound.on_start_processing();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}
	expect_frame_soon(Frame::from_mono(15.0).panned(0.5), &mut sound);
}

/// Tests that a `StreamingSound` can seek by an amount of time.
#[test]
fn seek_by() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().start_position(10.0),
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	handle.seek_by(5.0).unwrap();
	sound.on_start_processing();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}
	expect_frame_soon(Frame::from_mono(20.0).panned(0.5), &mut sound);
}

fn expect_frame_soon(expected_frame: Frame, sound: &mut StreamingSound) {
	const NUM_SAMPLES_TO_WAIT: usize = 10;
	for _ in 0..NUM_SAMPLES_TO_WAIT {
		let frame = sound.process(1.0);
		if frame == expected_frame {
			return;
		}
	}
	panic!(
		"Sound did not output frame with value {:?} within {} samples",
		expected_frame, NUM_SAMPLES_TO_WAIT
	);
}
