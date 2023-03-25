use std::{sync::Arc, time::Duration};

use crate::dsp::Frame;

use super::StaticSoundData;

#[test]
fn duration() {
	let static_sound = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new([Frame::from_mono(0.0); 4]),
		settings: Default::default(),
	};
	assert_eq!(static_sound.duration(), Duration::from_secs(4));
}
