use std::time::Duration;

use crate::{sound::Sound, Frame};

use super::StaticSound;

#[test]
fn duration() {
	let mut static_sound = StaticSound::from_frames(
		1,
		vec![
			Frame::from_mono(0.0),
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		],
		Default::default(),
	);
	assert_eq!(static_sound.duration(), Duration::from_secs(4));
}

#[test]
fn frame_at_position() {
	let mut static_sound = StaticSound::from_frames(
		1,
		vec![
			Frame::from_mono(0.0),
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		],
		Default::default(),
	);
	assert_eq!(static_sound.frame_at_position(-1.0), Frame::from_mono(0.0));
	assert_eq!(static_sound.frame_at_position(0.0), Frame::from_mono(0.0));
	assert_eq!(static_sound.frame_at_position(1.0), Frame::from_mono(1.0));
	assert_eq!(static_sound.frame_at_position(2.0), Frame::from_mono(2.0));
	assert_eq!(static_sound.frame_at_position(3.0), Frame::from_mono(3.0));
	assert_eq!(static_sound.frame_at_position(4.0), Frame::from_mono(0.0));
}
