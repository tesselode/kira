use std::{sync::Arc, time::Duration};

use atomic_arena::Arena;

use crate::{
	clock::{ClockId, ClockTime},
	dsp::Frame,
	sound::{
		static_sound::{PlaybackState, StaticSoundData, StaticSoundSettings},
		Sound,
	},
	tween::Tween,
	LoopBehavior,
};

/// Tests that a `StaticSound` will play all of its samples before finishing.
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
	assert_eq!(sound.state, PlaybackState::Playing);

	for i in 1..=3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
		assert!(!sound.finished());
		assert_eq!(sound.state, PlaybackState::Playing);
	}

	assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
	assert!(sound.finished());
	assert_eq!(sound.state, PlaybackState::Stopped);
}

/// Tests that a `StaticSound` correctly reports its playback state
/// to be queried by StaticSoundHandle::state.
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

/// Tests that a `StaticSound` correctly reports its playback state
/// to be queried by StaticSoundHandle::state.
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

/// Tests that a `StaticSound` fades out fully before pausing
/// and fades back in when resuming.
#[test]
#[allow(clippy::float_cmp)]
fn pauses_and_resumes_with_fades() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(1.0); 100]),
		settings: StaticSoundSettings::new(),
	};
	let (mut sound, mut handle) = data.split();

	sound.process(1.0);
	assert_eq!(sound.position, 1.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.pause(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.75).panned(0.5));
	assert_eq!(sound.position, 2.0);
	assert_eq!(sound.state, PlaybackState::Pausing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
	assert_eq!(sound.position, 3.0);
	assert_eq!(sound.state, PlaybackState::Pausing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.25).panned(0.5));
	assert_eq!(sound.position, 4.0);
	assert_eq!(sound.state, PlaybackState::Pausing);

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
		assert_eq!(sound.position, 4.0);
		assert_eq!(sound.state, PlaybackState::Paused);
	}

	handle
		.resume(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.25).panned(0.5));
	assert_eq!(sound.position, 5.0);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
	assert_eq!(sound.position, 6.0);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.75).panned(0.5));
	assert_eq!(sound.position, 7.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	for i in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(1.0).panned(0.5));
		assert_eq!(sound.position, i as f64 + 8.0);
		assert_eq!(sound.state, PlaybackState::Playing);
	}
}

/// Tests that a `StaticSound` stops and finishes after a fade-out.
#[test]
#[allow(clippy::float_cmp)]
fn stops_with_fade_out() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(1.0); 100]),
		settings: StaticSoundSettings::new(),
	};
	let (mut sound, mut handle) = data.split();

	sound.process(1.0);
	assert_eq!(sound.position, 1.0);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.stop(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.75).panned(0.5));
	assert_eq!(sound.position, 2.0);
	assert_eq!(sound.state, PlaybackState::Stopping);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
	assert_eq!(sound.position, 3.0);
	assert_eq!(sound.state, PlaybackState::Stopping);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.25).panned(0.5));
	assert_eq!(sound.position, 4.0);
	assert_eq!(sound.state, PlaybackState::Stopping);

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
		assert_eq!(sound.position, 4.0);
		assert_eq!(sound.state, PlaybackState::Stopped);
		assert!(sound.finished());
	}
}

/// Tests that a `StaticSound` will wait for its start clock time
/// when appropriate.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_start_time() {
	// create some fake ClockIds
	let mut dummy_arena = Arena::new(2);
	let key1 = dummy_arena.insert(()).unwrap();
	let key2 = dummy_arena.insert(()).unwrap();
	let clock_id_1 = ClockId(key1);
	let clock_id_2 = ClockId(key2);

	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(1.0); 100]),
		settings: StaticSoundSettings::new().start_time(ClockTime {
			clock: clock_id_1,
			ticks: 2,
		}),
	};
	let (mut sound, _) = data.split();

	// the sound should not be playing yet
	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0));
		assert_eq!(sound.position, 0.0);
	}

	// the sound is set to start at tick 2, so it should not
	// play yet
	sound.on_clock_tick(ClockTime {
		clock: clock_id_1,
		ticks: 1,
	});

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0));
		assert_eq!(sound.position, 0.0);
	}

	// this is a tick event for a different clock, so the
	// sound should not play yet
	sound.on_clock_tick(ClockTime {
		clock: clock_id_2,
		ticks: 2,
	});

	for _ in 0..3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(0.0));
		assert_eq!(sound.position, 0.0);
	}

	// the sound should start playing now
	sound.on_clock_tick(ClockTime {
		clock: clock_id_1,
		ticks: 2,
	});

	for i in 1..=3 {
		assert_eq!(sound.process(1.0), Frame::from_mono(1.0).panned(0.5));
		assert_eq!(sound.position, i as f64);
	}
}

/// Tests that a `StaticSound` can be started partway through the sound.
#[test]
#[allow(clippy::float_cmp)]
fn start_position() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new((0..10).map(|i| Frame::from_mono(i as f32)).collect()),
		settings: StaticSoundSettings::new().start_position(3.0),
	};
	let (mut sound, _) = data.split();

	assert_eq!(sound.position, 3.0);
	assert_eq!(sound.process(1.0), Frame::from_mono(3.0).panned(0.5));
}

/// Tests that a `StaticSound` can be played backwards.
#[test]
#[allow(clippy::float_cmp)]
fn reverse() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new((0..10).map(|i| Frame::from_mono(i as f32)).collect()),
		settings: StaticSoundSettings::new().reverse(true).start_position(2.0),
	};
	let (mut sound, _) = data.split();

	// start position should be from the end
	assert_eq!(sound.position, 8.0);
	assert_eq!(sound.process(1.0), Frame::from_mono(8.0).panned(0.5));
	// position should decrease over time
	assert_eq!(sound.position, 7.0);
	assert_eq!(sound.process(1.0), Frame::from_mono(7.0).panned(0.5));
}

/// Tests that a `StaticSound` properly obeys looping behavior when
/// playing forward.
#[test]
#[allow(clippy::float_cmp)]
fn loops_forward() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new((0..10).map(|i| Frame::from_mono(i as f32)).collect()),
		settings: StaticSoundSettings::new().loop_behavior(LoopBehavior {
			start_position: 3.0,
		}),
	};
	let (mut sound, _) = data.split();

	for i in 0..10 {
		assert_eq!(sound.position, i as f64);
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
	}

	assert_eq!(sound.position, 3.0);
	assert_eq!(sound.process(3.0), Frame::from_mono(3.0).panned(0.5));
	assert_eq!(sound.position, 6.0);
	assert_eq!(sound.process(3.0), Frame::from_mono(6.0).panned(0.5));
	assert_eq!(sound.position, 9.0);
	assert_eq!(sound.process(3.0), Frame::from_mono(9.0).panned(0.5));
	assert_eq!(sound.position, 5.0);
	assert_eq!(sound.process(3.0), Frame::from_mono(5.0).panned(0.5));
}

/// Tests that a `StaticSound` properly obeys looping behavior when
/// playing backward.
#[test]
#[allow(clippy::float_cmp)]
fn loops_backward() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new((0..10).map(|i| Frame::from_mono(i as f32)).collect()),
		settings: StaticSoundSettings::new()
			.loop_behavior(LoopBehavior {
				start_position: 3.0,
			})
			.reverse(true),
	};
	let (mut sound, _) = data.split();

	assert_eq!(sound.position, 10.0);
	assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));

	for i in (3..10).rev() {
		assert_eq!(sound.position, i as f64);
		assert_eq!(sound.process(1.0), Frame::from_mono(i as f32).panned(0.5));
	}

	assert_eq!(sound.position, 9.0);
	assert_eq!(sound.process(4.0), Frame::from_mono(9.0).panned(0.5));

	assert_eq!(sound.position, 5.0);
	assert_eq!(sound.process(4.0), Frame::from_mono(5.0).panned(0.5));
	assert_eq!(sound.position, 8.0);
	assert_eq!(sound.process(4.0), Frame::from_mono(8.0).panned(0.5));
}

/// Tests that the volume of a `StaticSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn volume() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(1.0); 10]),
		settings: StaticSoundSettings::new().volume(0.5),
	};
	let (mut sound, _) = data.split();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.5).panned(0.5));
}

/// Tests that the panning of a `StaticSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn panning() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(1.0); 10]),
		settings: StaticSoundSettings::new().panning(0.0),
	};
	let (mut sound, _) = data.split();

	assert_eq!(sound.process(1.0), Frame::from_mono(1.0).panned(0.0));
}

/// Tests that the playback rate of a `StaticSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn playback_rate() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new((0..10).map(|i| Frame::from_mono(i as f32)).collect()),
		settings: StaticSoundSettings::new().playback_rate(2.0),
	};
	let (mut sound, _) = data.split();

	assert_eq!(sound.process(1.0), Frame::from_mono(0.0).panned(0.5));
	assert_eq!(sound.position, 2.0);
	assert_eq!(sound.process(1.0), Frame::from_mono(2.0).panned(0.5));
}

/// Tests that a `StaticSound` outputs interpolated samples when
/// its playback position is between samples.
#[test]
fn interpolates_samples() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![
			Frame::from_mono(0.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(-10.0),
		]),
		settings: Default::default(),
	};
	let (mut sound, _) = data.split();

	assert_eq!(sound.process(0.5), Frame::from_mono(0.0).panned(0.5));
	// at sample 0.5, the output should be somewhere between 0 and 1.
	// i don't care what exactly, that's up the to the interpolation algorithm.
	let frame = sound.process(5.0);
	assert!(frame.left > 0.0 && frame.left < 1.0);
	// at sample 5.5, the output should be between 1 and -10.
	let frame = sound.process(1.0);
	assert!(frame.left < 0.0 && frame.left > -10.0);
}

/// Tests that a `StaticSound` outputs interpolated samples correctly
/// when looping.
#[test]
fn interpolates_samples_when_looping() {
	let data = StaticSoundData {
		sample_rate: 1,
		frames: Arc::new(vec![Frame::from_mono(10.0), Frame::from_mono(9.0)]),
		settings: StaticSoundSettings::new().loop_behavior(LoopBehavior {
			start_position: 0.0,
		}),
	};
	let (mut sound, _) = data.split();
	sound.process(1.5);
	// because we're looping back to the first sample, which is 10.0,
	// the interpolated sample should be be tween 9.0 and 10.0
	let frame = sound.process(1.0);
	assert!(frame.left > 9.0 && frame.left < 10.0);
}
