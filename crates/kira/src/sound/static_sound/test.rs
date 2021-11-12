use std::{sync::Arc, time::Duration};

use crate::dsp::Frame;

use super::data::{Samples, StaticSoundData};

#[test]
fn duration() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		samples: Arc::new(Samples::I16Mono(vec![0; 4])),
		settings: Default::default(),
	};
	assert_eq!(static_sound.duration(), Duration::from_secs(4));
}

#[test]
fn frame_at_position() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		samples: Arc::new(Samples::F32Mono(vec![0.0, 1.0, 2.0, 3.0])),
		settings: Default::default(),
	};
	assert_eq!(static_sound.frame_at_position(-1.0), Frame::from_mono(0.0));
	assert_eq!(static_sound.frame_at_position(0.0), Frame::from_mono(0.0));
	assert_eq!(static_sound.frame_at_position(1.0), Frame::from_mono(1.0));
	assert_eq!(static_sound.frame_at_position(2.0), Frame::from_mono(2.0));
	assert_eq!(static_sound.frame_at_position(3.0), Frame::from_mono(3.0));
	assert_eq!(static_sound.frame_at_position(4.0), Frame::from_mono(0.0));
}
