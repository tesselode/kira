use std::sync::Arc;

use crate::{
	dsp::Frame,
	sound::{
		static_sound::{PlaybackState, StaticSoundData, StaticSoundSettings},
		Sound,
	},
};

#[test]
fn plays_all_samples() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		]),
		settings: StaticSoundSettings::new(),
	};
	let (mut sound, _) = data.split();

	assert!(!sound.finished());
	assert_eq!(sound.state(), PlaybackState::Playing);

	for i in 1..=3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
		assert!(!sound.finished());
		assert_eq!(sound.state(), PlaybackState::Playing);
	}

	assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
	assert!(sound.finished());
	assert_eq!(sound.state(), PlaybackState::Stopped);
}

#[test]
fn reports_playback_state() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(0.0); 2]),
		settings: StaticSoundSettings::new(),
	};
	let (mut sound, handle) = data.split();

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

#[test]
#[allow(clippy::float_cmp)]
fn reports_playback_position() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(0.0); 2]),
		settings: StaticSoundSettings::new(),
	};
	let (mut sound, handle) = data.split();

	assert_eq!(handle.position(), 0.0);

	for i in 1..=2 {
		sound.process(1.0);
		sound.on_start_processing();
		assert_eq!(handle.position(), i as f64);
	}

	sound.process(1.0);
	sound.on_start_processing();
	assert_eq!(handle.position(), 2.0);
}
