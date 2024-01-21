use std::time::Duration;

use crate::{
	clock::{
		clock_info::{ClockInfo, ClockInfoProvider, MockClockInfoProviderBuilder},
		ClockTime,
	},
	dsp::Frame,
	modulator::value_provider::{MockModulatorValueProviderBuilder, ModulatorValueProvider},
	sound::{
		wrapper::{SoundWrapper, SoundWrapperShared},
		CommonSoundSettings, PlaybackState, Sound,
	},
	tween::Tween,
	StartTime, Volume,
};

/// Tests that a `SoundWrapper` fades out fully before pausing
/// and fades back in when resuming.
#[test]
#[allow(clippy::float_cmp)]
fn pauses_and_resumes_with_fades() {
	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: Default::default(),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	expect_frame_soon(
		Frame::from_mono(1.0),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Playing);

	sound_wrapper.pause(Tween {
		duration: Duration::from_secs(4),
		..Default::default()
	});
	sound_wrapper.on_start_processing();
	assert_eq!(sound_wrapper.state, PlaybackState::Pausing);

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade out soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(
		sound_wrapper.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(
		sound_wrapper.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5)
	);

	// make sure the sound is paused
	sound_wrapper.on_start_processing();
	for _ in 0..10 {
		assert_eq!(
			sound_wrapper.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0).panned(0.5)
		);
		sound_wrapper.on_start_processing();
		assert_eq!(sound_wrapper.state, PlaybackState::Paused);
	}

	sound_wrapper.resume(Tween {
		duration: Duration::from_secs(4),
		..Default::default()
	});
	sound_wrapper.on_start_processing();

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade back in soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Playing);
	assert_eq!(
		sound_wrapper.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Playing);
	assert_eq!(
		sound_wrapper.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Playing);

	// the sound should be playing normally now
	for _ in 0..3 {
		assert_eq!(
			sound_wrapper.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(1.0).panned(0.5)
		);
		assert_eq!(sound_wrapper.state, PlaybackState::Playing);
	}
}

/// Tests that a `SoundWrapper` stops and finishes after a fade-out.
#[test]
#[allow(clippy::float_cmp)]
fn stops_with_fade_out() {
	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: Default::default(),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	expect_frame_soon(
		Frame::from_mono(1.0),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Playing);

	sound_wrapper.stop(Tween {
		duration: Duration::from_secs(4),
		..Default::default()
	});
	sound_wrapper.on_start_processing();
	assert_eq!(sound_wrapper.state, PlaybackState::Stopping);

	// allow for a few samples of delay because of the resampling, but the
	// sound should fade out soon.
	expect_frame_soon(
		Frame::from_mono(Volume::Decibels(-15.0).as_amplitude() as f32).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(
		sound_wrapper.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-30.0).as_amplitude() as f32).panned(0.5)
	);
	assert_eq!(
		sound_wrapper.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(Volume::Decibels(-45.0).as_amplitude() as f32).panned(0.5)
	);

	// the sound should be stopped
	sound_wrapper.on_start_processing();
	for _ in 0..3 {
		assert_eq!(
			sound_wrapper.process(
				1.0,
				&MockClockInfoProviderBuilder::new(0).build(),
				&MockModulatorValueProviderBuilder::new(0).build()
			),
			Frame::from_mono(0.0).panned(0.5)
		);
		sound_wrapper.on_start_processing();
		assert_eq!(sound_wrapper.state, PlaybackState::Stopped);
		assert!(sound_wrapper.finished());
	}
}

/// Tests that a `SoundWrapper` with a delayed start time waits for
/// that time before it begins tweening.
#[test]
#[allow(clippy::float_cmp)]
fn waits_for_delay() {
	let clock_info_provider = MockClockInfoProviderBuilder::new(0).build();
	let modulator_value_provider = MockModulatorValueProviderBuilder::new(0).build();

	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: StartTime::Delayed(Duration::from_secs(100)),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	// sound should not be playing yet
	for _ in 0..100 {
		assert_eq!(
			sound_wrapper.process(1.0, &clock_info_provider, &modulator_value_provider),
			Frame::from_mono(0.0)
		);
	}

	// the sound should start now
	expect_frame_soon(
		Frame::from_mono(1.0),
		&mut sound_wrapper,
		&clock_info_provider,
		&modulator_value_provider,
	);
}

/// Tests that a `SoundWrapper` will wait for its start clock time
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

	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id_1,
				ticks: 2,
			}),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	// the sound should not be playing yet
	for _ in 0..3 {
		assert_eq!(
			sound_wrapper.process(
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
			sound_wrapper.process(
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
			sound_wrapper.process(
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
	expect_frame_soon(
		Frame::from_mono(1.0),
		&mut sound_wrapper,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
}

/// Tests that a `SoundWrapper` will stop (allowing it to be removed)
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

	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id,
				ticks: 2,
			}),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	sound_wrapper.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Playing);

	// the clock is removed
	let clock_info_provider = MockClockInfoProviderBuilder::new(1).build();

	sound_wrapper.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	// the sound needs one extra process call to go from Stopping to Stopped
	sound_wrapper.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert_eq!(sound_wrapper.state, PlaybackState::Stopped);
}

/// Tests that a `SoundWrapper` can be paused, resumed, and
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

	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: StartTime::ClockTime(ClockTime {
				clock: clock_id,
				ticks: 2,
			}),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	sound_wrapper.pause(Tween {
		duration: Duration::from_secs(0),
		..Default::default()
	});
	sound_wrapper.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(sound_wrapper.state == PlaybackState::Paused);

	sound_wrapper.resume(Tween {
		duration: Duration::from_secs(0),
		..Default::default()
	});
	sound_wrapper.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(sound_wrapper.state == PlaybackState::Playing);

	sound_wrapper.stop(Tween {
		duration: Duration::from_secs(0),
		..Default::default()
	});
	sound_wrapper.process(
		1.0,
		&clock_info_provider,
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	assert!(sound_wrapper.state == PlaybackState::Stopped);
}

/// Tests that the volume of a `SoundWrapper` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn volume() {
	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: Default::default(),
			volume: 0.5.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	expect_frame_soon(
		Frame::from_mono(0.5).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
}

/// Tests that the volume of a `SoundWrapper` can be changed
/// after the sound is started.
#[test]
fn set_volume() {
	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: Default::default(),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	expect_frame_soon(
		Frame::from_mono(1.0).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	sound_wrapper.set_volume(
		0.5.into(),
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	expect_frame_soon(
		Frame::from_mono(0.5).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
}

/// Tests that the panning of a `SoundWrapper` can be adjusted.
#[test]
#[allow(clippy::float_cmp)]
fn panning() {
	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: Default::default(),
			volume: 1.0.into(),
			panning: 0.0.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	expect_frame_soon(
		Frame::from_mono(1.0).panned(0.0),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
}

/// Tests that the panning of a `SoundWrapper` can be changed
/// after the sound is started.
#[test]
fn set_panning() {
	let shared = SoundWrapperShared::new();
	let mut sound_wrapper = SoundWrapper::new(
		Box::new(MockSound),
		CommonSoundSettings {
			start_time: Default::default(),
			volume: 1.0.into(),
			panning: 0.5.into(),
			output_destination: Default::default(),
			fade_in_tween: None,
		},
		shared,
	);

	expect_frame_soon(
		Frame::from_mono(1.0).panned(0.5),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
	sound_wrapper.set_panning(
		0.0.into(),
		Tween {
			duration: Duration::ZERO,
			..Default::default()
		},
	);
	expect_frame_soon(
		Frame::from_mono(1.0).panned(0.0),
		&mut sound_wrapper,
		&MockClockInfoProviderBuilder::new(0).build(),
		&MockModulatorValueProviderBuilder::new(0).build(),
	);
}

struct MockSound;

impl Sound for MockSound {
	fn sample_rate(&self) -> f64 {
		1.0
	}

	fn process(
		&mut self,
		_clock_info_provider: &ClockInfoProvider,
		_modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		Frame::from_mono(1.0)
	}

	fn finished(&self) -> bool {
		false
	}
}

fn expect_frame_soon(
	expected_frame: Frame,
	sound_wrapper: &mut SoundWrapper,
	clock_info_provider: &ClockInfoProvider,
	modulator_value_provider: &ModulatorValueProvider,
) {
	const NUM_SAMPLES_TO_WAIT: usize = 10;
	for _ in 0..NUM_SAMPLES_TO_WAIT {
		let frame = sound_wrapper.process(1.0, clock_info_provider, modulator_value_provider);
		if frame == expected_frame {
			return;
		}
	}
	panic!(
		"Sound did not output frame with value {:?} within {} samples",
		expected_frame, NUM_SAMPLES_TO_WAIT
	);
}
