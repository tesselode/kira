use std::{sync::Arc, time::Duration};

use crate::dsp::Frame;

use super::StaticSoundData;

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
fn sliced_duration() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
		slice: None,
	};
	assert_eq!(static_sound.duration(), Duration::from_secs(4));

	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
		slice: Some((2, 3)),
	};
	assert_eq!(static_sound.duration(), Duration::from_secs(1));
}

#[test]
fn slice() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: (0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: Default::default(),
		slice: None,
	}
	.slice(3.0..6.0);
	for i in 0..3 {
		assert_eq!(
			static_sound.frame_at_index(i),
			Some(Frame::from_mono(i as f32 + 3.0))
		);
	}
	assert!(static_sound.frame_at_index(3).is_none());
}
