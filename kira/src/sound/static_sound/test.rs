use std::time::Duration;

use crate::{dsp::Frame, sound::static_sound::Samples};

use super::StaticSound;

#[test]
fn duration() {
	let static_sound = StaticSound::new(1, Samples::I16Mono(vec![0; 4]));
	assert_eq!(static_sound.duration(), Duration::from_secs(4));
}

#[test]
fn frame_at_position() {
	let static_sound = StaticSound::new(1, Samples::F32Mono(vec![0.0, 1.0, 2.0, 3.0]));
	assert_eq!(static_sound.frame_at_position(-1.0), Frame::from_mono(0.0));
	assert_eq!(static_sound.frame_at_position(0.0), Frame::from_mono(0.0));
	assert_eq!(static_sound.frame_at_position(1.0), Frame::from_mono(1.0));
	assert_eq!(static_sound.frame_at_position(2.0), Frame::from_mono(2.0));
	assert_eq!(static_sound.frame_at_position(3.0), Frame::from_mono(3.0));
	assert_eq!(static_sound.frame_at_position(4.0), Frame::from_mono(0.0));
}
