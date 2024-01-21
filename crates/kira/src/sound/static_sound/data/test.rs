use std::{sync::Arc, time::Duration};

use crate::dsp::Frame;

use super::StaticSoundData;

#[test]
fn num_frames() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
		slice: None,
	};
	assert_eq!(static_sound.num_frames(), 4);
}

#[test]
fn duration() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
		slice: None,
	};
	assert_eq!(static_sound.duration(), Duration::from_secs(4));
}

#[test]
fn frame() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: (0..10)
			.map(|i| Frame::from_mono(i as f32))
			.collect::<Vec<_>>()
			.into(),
		settings: Default::default(),
		slice: None,
	};
	for i in 0..10 {
		assert_eq!(static_sound.frame(i), Frame::from_mono(i as f32));
	}
}

#[test]
fn sliced_num_frames() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
		slice: Some((0, 2)),
	};
	assert_eq!(static_sound.num_frames(), 2);
}

#[test]
fn sliced_duration() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
		slice: Some((0, 2)),
	};
	assert_eq!(static_sound.duration(), Duration::from_secs(2));
}

#[test]
fn sliced_frame() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: (0..10)
			.map(|i| Frame::from_mono(i as f32))
			.collect::<Vec<_>>()
			.into(),
		settings: Default::default(),
		slice: Some((3, 5)),
	};
	for i in 0..2 {
		assert_eq!(static_sound.frame(i), Frame::from_mono(3.0 + i as f32));
	}
	assert_eq!(static_sound.frame(-1), Frame::ZERO);
	assert_eq!(static_sound.frame(2), Frame::ZERO);
}
