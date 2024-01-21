use std::time::Duration;

use crate::{
	clock::{
		clock_info::{ClockInfo, MockClockInfoProviderBuilder},
		ClockTime,
	},
	dsp::Frame,
	modulator::value_provider::MockModulatorValueProviderBuilder,
	sound::{
		streaming::{decoder::mock::MockDecoder, StreamingSoundData, StreamingSoundSettings},
		PlaybackState, Sound,
	},
	tween::Tween,
	StartTime, Volume,
};

use super::{decode_scheduler::NextStep, StreamingSound};

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
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert!(!sound.finished());

	for i in 1..=3 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
		assert!(!sound.finished());
	}

	// give some time for the resample buffer to empty. in the meantime we should
	// get silent output.
	for _ in 0..10 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0).panned(0.5)
		);
	}

	// the sound should be finished and stopped by now
	assert!(sound.finished());
	assert_eq!(sound.state, PlaybackState::Stopped);
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
	let (mut sound, handle, mut scheduler) = data.split().unwrap();

	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::ZERO.panned(0.5)
		);
		sound.on_start_processing();
		assert_eq!(handle.position(), 0.0);
	}

	for _ in 0..3 {
		scheduler.run().unwrap();
	}

	for i in 1..=3 {
		assert_eq!(handle.position(), (i - 1) as f64);
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
		sound.on_start_processing();
	}

	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::ZERO.panned(0.5)
		);
		sound.on_start_processing();
		assert_eq!(handle.position(), 2.0);
	}
}

/// Tests that a `StreamingSound` correctly reports its playback state
/// to be queried by StreamingSoundHandle::state.
#[test]
fn reports_playback_state() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.0); 10])),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for _ in 0..20 {
		assert_eq!(handle.state(), sound.state);
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
		);
	}
}

/// Tests that a `StreamingSound` correctly reports its playback state
/// to be queried by StreamingSoundHandle::state.
#[test]
#[allow(clippy::float_cmp)]
fn reports_playback_position() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.0); 10])),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for i in 0..20 {
		assert_eq!(handle.position(), i.clamp(0, 9) as f64);
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
		);
		sound.on_start_processing();
	}
}

/// Tests that a `StreamingSound` fades out fully before pausing
/// and fades back in when resuming.
#[test]
#[allow(clippy::float_cmp)]
fn pauses_and_resumes_with_fades() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 100])),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.pause(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();
	assert_eq!(sound.state, PlaybackState::Pausing);

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade out soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5),
		&mut sound,
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5)
	);
	sound.process(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);

	sound.on_start_processing();
	let position = handle.position();
	for _ in 0..10 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0).panned(0.5)
		);
		sound.on_start_processing();
		assert_eq!(handle.position(), position);
		assert_eq!(sound.state, PlaybackState::Paused);
	}

	handle
		.resume(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade back in soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5),
		&mut sound,
	);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(sound.state, PlaybackState::Playing);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(sound.state, PlaybackState::Playing);

	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(1.0).panned(0.5)
		);
		assert_eq!(sound.state, PlaybackState::Playing);
	}
}

/// Tests that a `StreamingSound` stops and finishes after a fade-out.
#[test]
#[allow(clippy::float_cmp)]
fn stops_with_fade_out() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 100])),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound.state, PlaybackState::Playing);

	handle
		.stop(Tween {
			duration: Duration::from_secs(4),
			..Default::default()
		})
		.unwrap();
	sound.on_start_processing();
	assert_eq!(sound.state, PlaybackState::Stopping);

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade out soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5),
		&mut sound,
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5)
	);
	sound.process(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);

	sound.on_start_processing();
	let position = handle.position();
	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0).panned(0.5)
		);
		sound.on_start_processing();
		assert_eq!(handle.position(), position);
		assert_eq!(sound.state, PlaybackState::Stopped);
		assert!(sound.finished());
	}
}

/// Tests that a `StreamingSound` will wait for its start clock time
/// when appropriate.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_start_time() {
	let (clock_info_provider, clock_id_1) = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		let clock_id_1 = builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		(builder.build(), clock_id_1)
	};

	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(1..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().start_time(ClockTime {
			clock: clock_id_1,
			ticks: 2,
		}),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	// the sound should not be playing yet
	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&clock_info_provider,
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0)
		);
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 1,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	// the sound is set to start at tick 2, so it should not
	// play yet
	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&clock_info_provider,
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0)
		);
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 1,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 2,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	// a different clock reached tick 2, so the sound should
	// not play yet
	for _ in 0..3 {
		assert_eq!(
			sound.process(
				1.0,
				&clock_info_provider,
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0)
		);
	}

	let clock_info_provider = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 2,
				fractional_position: 0.0,
			})
			.unwrap();
		builder
			.add(ClockInfo {
				ticking: true,
				ticks: 2,
				fractional_position: 0.0,
			})
			.unwrap();
		builder.build()
	};

	// the sound should start playing now
	for i in 1..10 {
		assert_eq!(
			sound.process(
				1.0,
				&clock_info_provider,
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
	}
}

/// Tests that a `StreamingSound` will stop (allowing it to be removed)
/// if it's waiting on a clock that no longer exists before it can
/// start.
///
/// Without this behavior, a sound in this situation would be in limbo
/// forever with no way to free up its resources.
#[test]
fn stops_if_depending_on_missing_clock() {
	let (clock_info_provider, clock_id) = {
		let mut builder = MockClockInfoProviderBuilder::new(1);
		let clock_id = builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		(builder.build(), clock_id)
	};

	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(1..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().start_time(ClockTime {
			clock: clock_id,
			ticks: 2,
		}),
		slice: None,
	};
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	sound.on_start_processing();
	assert_eq!(handle.state(), PlaybackState::Playing);

	// the clock is removed
	let clock_info_provider = MockClockInfoProviderBuilder::new(1).build();

	sound.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	sound.on_start_processing();
	assert_eq!(handle.state(), PlaybackState::Stopped);
}

/// Tests that a `StreamingSound` can be paused, resumed, and
/// stopped immediately even if playback is waiting for a clock
/// time to start.
#[test]
fn immediate_pause_resume_and_stop_with_clock_start_time() {
	let (clock_info_provider, clock_id) = {
		let mut builder = MockClockInfoProviderBuilder::new(2);
		let clock_id = builder
			.add(ClockInfo {
				ticking: true,
				ticks: 0,
				fractional_position: 0.0,
			})
			.unwrap();
		(builder.build(), clock_id)
	};

	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(1..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().start_time(StartTime::ClockTime(ClockTime {
			clock: clock_id,
			ticks: 2,
		})),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.pause(Tween {
		duration: Duration::from_secs(0),
		..Default::default()
	});
	sound.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(sound.state == PlaybackState::Paused);

	sound.resume(Tween {
		duration: Duration::from_secs(0),
		..Default::default()
	});
	sound.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(sound.state == PlaybackState::Playing);

	sound.stop(Tween {
		duration: Duration::from_secs(0),
		..Default::default()
	});
	sound.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(sound.state == PlaybackState::Stopped);
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
	let (mut sound, handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for i in 3..=6 {
		assert_eq!(handle.position(), i as f64);
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
		sound.on_start_processing();
	}
}

/// Tests that a `StreamingSound` properly obeys looping behavior when
/// playing forward.
#[test]
#[allow(clippy::float_cmp)]
fn loops_forward() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().loop_region(Some((3.0..6.0).into())),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	for i in 0..6 {
		assert_eq!(
			sound.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(i as f32).panned(0.5)
		);
	}

	assert_eq!(
		sound.process(
			2.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(3.0).panned(0.5)
	);
	assert_eq!(
		sound.process(
			2.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(5.0).panned(0.5)
	);
	assert_eq!(
		sound.process(
			2.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(4.0).panned(0.5)
	);
	assert_eq!(
		sound.process(
			2.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(3.0).panned(0.5)
	);
}

/// Tests that the volume of a `StreamingSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn volume() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 10])),
		settings: StreamingSoundSettings::new().volume(0.5),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.5).panned(0.5)
	);
}

/// Tests that the volume of a `StreamingSound` can be changed
/// after the sound is started.
#[test]
fn set_volume() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 100])),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(1.0).panned(0.5)
	);
	handle
		.set_volume(
			0.5,
			Tween {
				duration: Duration::ZERO,
				..Default::default()
			},
		)
		.unwrap();
	sound.on_start_processing();
	expect_frame_soon(Frame::from_mono(0.5).panned(0.5), &mut sound);
}

/// Tests that the panning of a `StreamingSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn panning() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 10])),
		settings: StreamingSoundSettings::new().panning(0.0),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(1.0).panned(0.0)
	);
}

/// Tests that the panning of a `StreamingSound` can be changed
/// after the sound is started.
#[test]
fn set_panning() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(1.0); 100])),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(1.0).panned(0.5)
	);
	handle
		.set_panning(
			0.0,
			Tween {
				duration: Duration::ZERO,
				..Default::default()
			},
		)
		.unwrap();
	sound.on_start_processing();
	expect_frame_soon(Frame::from_mono(1.0).panned(0.0), &mut sound);
}

/// Tests that the playback rate of a `StreamingSound` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn playback_rate() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new().playback_rate(2.0),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.0).panned(0.5)
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(2.0).panned(0.5)
	);
}

/// Tests that the playback rate of a `StreamingSound` can be adjusted after
/// it's started.
#[test]
#[allow(clippy::float_cmp)]
fn set_playback_rate() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(
			(0..100).map(|i| Frame::from_mono(i as f32)).collect(),
		)),
		settings: StreamingSoundSettings::new(),
		slice: None,
	};
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.0).panned(0.5)
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(1.0).panned(0.5)
	);

	handle
		.set_playback_rate(
			2.0,
			Tween {
				duration: Duration::ZERO,
				..Default::default()
			},
		)
		.unwrap();
	sound.on_start_processing();

	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(2.0).panned(0.5)
	);
	assert_eq!(
		sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(4.0).panned(0.5)
	);
}

/// Tests that a `StreamingSound` outputs interpolated samples when
/// its playback position is between samples.
#[test]
fn interpolates_samples() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![
			Frame::from_mono(0.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(1.0),
			Frame::from_mono(-10.0),
		])),
		settings: Default::default(),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	assert_eq!(
		sound.process(
			0.5,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.0).panned(0.5)
	);
	// at sample 0.5, the output should be somewhere between 0 and 1.
	// i don't care what exactly, that's up the to the interpolation algorithm.
	let frame = sound.process(
		5.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(frame.left > 0.0 && frame.left < 1.0);
	// at sample 5.5, the output should be between 1 and -10.
	let frame = sound.process(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(frame.left < 0.0 && frame.left > -10.0);
}

/// Tests that a `StreamingSound` outputs interpolated samples correctly
/// when looping.
#[test]
fn interpolates_samples_when_looping() {
	let data = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![
			Frame::from_mono(10.0),
			Frame::from_mono(9.0),
		])),
		settings: StreamingSoundSettings::new().loop_region(Some((..).into())),
		slice: None,
	};
	let (mut sound, _, mut scheduler) = data.split().unwrap();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}

	sound.process(
		1.5,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	// because we're looping back to the first sample, which is 10.0,
	// the interpolated sample should be be tween 9.0 and 10.0
	let frame = sound.process(
		1.0,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(frame.left > 9.0 && frame.left < 10.0);
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
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();

	handle.seek_to(15.0).unwrap();
	sound.on_start_processing();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}
	expect_frame_soon(Frame::from_mono(15.0).panned(0.5), &mut sound);
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
	let (mut sound, mut handle, mut scheduler) = data.split().unwrap();
	handle.seek_by(5.0).unwrap();
	sound.on_start_processing();
	while matches!(scheduler.run().unwrap(), NextStep::Continue) {}
	expect_frame_soon(Frame::from_mono(20.0).panned(0.5), &mut sound);
}

fn expect_frame_soon(expected_frame: Frame, sound: &mut StreamingSound) {
	const NUM_SAMPLES_TO_WAIT: usize = 10;
	for _ in 0..NUM_SAMPLES_TO_WAIT {
		let frame = sound.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build(),
		);
		if frame == expected_frame {
			return;
		}
	}
	panic!(
		"Sound did not output frame with value {:?} within {} samples",
		expected_frame, NUM_SAMPLES_TO_WAIT
	);
}
