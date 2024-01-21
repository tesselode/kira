use crate::{
	clock::clock_info::MockClockInfoProviderBuilder,
	dsp::Frame,
	modulator::value_provider::MockModulatorValueProviderBuilder,
	sound::{
		streaming::{decoder::mock::MockDecoder, StreamingSoundData, StreamingSoundSettings},
		Sound,
	},
};

use super::decode_scheduler::NextStep;

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
		slice: None,
	};
	let (mut sound, mut scheduler) = data.split_without_handle().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert!(!sound.finished());

	for i in 1..=3 {
		assert!(!sound.finished());
		assert_eq!(
			sound.process(
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
	}
	assert!(sound.finished());
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
		slice: None,
	};
	let (mut sound, mut scheduler) = data.split_without_handle().unwrap();

	for _ in 0..3 {
		assert_eq!(
			sound.process(
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::ZERO.panned(0.5)
		);
	}

	for _ in 0..3 {
		scheduler.run().unwrap();
	}

	for i in 1..=3 {
		assert_eq!(
			sound.process(
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
	}

	for _ in 0..3 {
		assert_eq!(
			sound.process(
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::ZERO.panned(0.5)
		);
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
		slice: None,
	};
	let (mut sound, mut scheduler) = data.split_without_handle().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for i in 3..=6 {
		assert_eq!(
			sound.process(
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
	}
}

/// Tests that a `StreamingSound` can seek to a position.
#[test]
fn seek_to() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, mut scheduler) = data.split_without_handle().unwrap();

	scheduler.seek_to(15.0).unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}
	assert_eq!(
		sound.process(
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(15.0)
	);
}

/// Tests that a `StreamingSound` can seek by an amount of time.
#[test]
fn seek_by() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().start_position(10.0),
		slice: None,
	};
	let (mut sound, mut scheduler) = data.split_without_handle().unwrap();
	scheduler.seek_by(5.0).unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}
	assert_eq!(
		sound.process(
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(15.0)
	);
}
