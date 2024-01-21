use std::sync::Arc;

use ringbuf::HeapRb;

use crate::{
	clock::clock_info::MockClockInfoProviderBuilder,
	dsp::Frame,
	modulator::value_provider::MockModulatorValueProviderBuilder,
	sound::{
		static_sound::{StaticSoundData, StaticSoundSettings},
		Sound,
	},
};

use super::StaticSound;

/// Tests that a `StaticSound` will play all of its samples before finishing.
#[test]
fn plays_all_samples() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		]),
		settings: StaticSoundSettings::new(),
		slice: None,
	};
	let (_, heap_consumer) = HeapRb::new(1).split();
	let mut sound = StaticSound::new(data, heap_consumer);

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

/// Tests that a `StaticSound` can be started in the middle.
#[test]
#[allow(clippy::float_cmp)]
fn start_position() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: (0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: StaticSoundSettings::new().start_position(3.0),
		slice: None,
	};
	let (_, heap_consumer) = HeapRb::new(1).split();
	let mut sound = StaticSound::new(data, heap_consumer);

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

/// Tests that starting a `StaticSound` past the end of the sound
/// will not cause a panic.
#[test]
#[allow(clippy::float_cmp)]
fn out_of_bounds_start_position() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: (0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: StaticSoundSettings::new().start_position(15.0),
		slice: None,
	};
	let (_, heap_consumer) = HeapRb::new(1).split();
	let mut sound = StaticSound::new(data, heap_consumer);
	sound.process(
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
}

/// Tests that a `StaticSound` can seek to a position.
#[test]
fn seek_to() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: (0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: StaticSoundSettings::new(),
		slice: None,
	};
	let (_, heap_consumer) = HeapRb::new(1).split();
	let mut sound = StaticSound::new(data, heap_consumer);

	sound.seek_to(25.0);
	assert_eq!(
		sound.process(
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(25.0)
	);
}

/// Tests that a `StaticSound` can seek by an amount of time.
#[test]
fn seek_by() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: (0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: StaticSoundSettings::new().start_position(10.0),
		slice: None,
	};
	let (_, heap_consumer) = HeapRb::new(1).split();
	let mut sound = StaticSound::new(data, heap_consumer);

	sound.seek_by(5.0);
	assert_eq!(
		sound.process(
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(15.0)
	);
}

/// Tests that a `StaticSound` can play in reverse.
#[test]
fn reverse() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: (0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: StaticSoundSettings::new().reverse(true),
		slice: None,
	};
	let (_, heap_consumer) = HeapRb::new(1).split();
	let mut sound = StaticSound::new(data, heap_consumer);

	for i in (4..=9).rev() {
		assert_eq!(
			sound.process(
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
	}
}
