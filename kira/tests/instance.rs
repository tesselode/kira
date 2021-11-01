use std::{error::Error, time::Duration};

use kira::{
	manager::{AudioManager, MockBackend},
	parameter::Tween,
	sound::{
		instance::{InstanceSettings, InstanceState},
		static_sound::StaticSound,
	},
	Frame, LoopBehavior,
};

fn assert_frame_approximate_eq(a: Frame, b: Frame) {
	const ERROR_THRESHOLD: f32 = 1.0e-6;
	if (a.left - b.left).abs() > ERROR_THRESHOLD || (a.right - b.right).abs() > ERROR_THRESHOLD {
		assert_eq!(a, b);
	}
}

#[test]
fn plays_all_samples_of_a_sound() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		],
		Default::default(),
	))?;
	sound.play(Default::default())?;
	manager.backend_mut().on_start_processing(0.0);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(1.0));
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(2.0));
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(3.0));
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(0.0));
	Ok(())
}

#[test]
fn stops_after_sound_is_finished() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		],
		Default::default(),
	))?;
	let instance = sound.play(Default::default())?;
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.state(), InstanceState::Playing);
	for _ in 0..3 {
		manager.backend_mut().process();
		assert_eq!(instance.state(), InstanceState::Playing);
	}
	manager.backend_mut().process();
	assert_eq!(instance.state(), InstanceState::Stopped);
	Ok(())
}

#[test]
#[allow(clippy::float_cmp)]
fn reports_playback_position() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![
			Frame::from_mono(1.0),
			Frame::from_mono(2.0),
			Frame::from_mono(3.0),
		],
		Default::default(),
	))?;
	let instance = sound.play(Default::default())?;
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 0.0);
	for i in 1..4 {
		manager.backend_mut().process();
		manager.backend_mut().on_start_processing(0.0);
		assert_eq!(instance.position(), i as f64);
	}
	Ok(())
}

#[test]
#[allow(clippy::float_cmp)]
fn allows_customizing_playback_rate() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![Frame::from_mono(0.0); 10],
		Default::default(),
	))?;
	// forward playback
	let instance = sound.play(InstanceSettings::new().playback_rate(3.0))?;
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 0.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 3.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 6.0);
	// positive playback rate, reverse on
	let instance = sound.play(InstanceSettings::new().playback_rate(3.0).reverse(true))?;
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 10.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 7.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 4.0);
	// negative playback rate, reverse off
	let instance = sound.play(
		InstanceSettings::new()
			.playback_rate(-3.0)
			.start_position(10.0),
	)?;
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 10.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 7.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 4.0);
	Ok(())
}

#[test]
#[allow(clippy::float_cmp)]
fn loops_when_playing_forward() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![Frame::from_mono(0.0); 10],
		Default::default(),
	))?;
	let instance = sound.play(
		InstanceSettings::new()
			.loop_behavior(LoopBehavior {
				start_position: 5.0,
			})
			.playback_rate(3.0),
	)?;
	manager.backend_mut().on_start_processing(0.0);
	for _ in 0..4 {
		manager.backend_mut().process();
	}
	// instance position should now be 12.0, which should loop back to 7.0
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 7.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	// the current behavior is to only loop back when the position exceeds
	// the duration of the sound
	assert_eq!(instance.position(), 10.0);
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 8.0);
	Ok(())
}

#[test]
#[allow(clippy::float_cmp)]
fn loops_when_playing_backward() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![Frame::from_mono(0.0); 10],
		Default::default(),
	))?;
	let instance = sound.play(
		InstanceSettings::new()
			.loop_behavior(LoopBehavior {
				start_position: 5.0,
			})
			.playback_rate(3.0)
			.reverse(true),
	)?;
	manager.backend_mut().on_start_processing(0.0);
	for _ in 0..2 {
		manager.backend_mut().process();
	}
	// instance position should now be 4.0, which should loop back to 9.0
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 9.0);
	Ok(())
}

#[test]
fn stops_with_fade_out() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![Frame::from_mono(4.0); 10],
		Default::default(),
	))?;
	let mut instance = sound.play(Default::default())?;
	manager.backend_mut().on_start_processing(0.0);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(4.0));
	assert_eq!(instance.state(), InstanceState::Playing);
	instance.stop(Tween {
		duration: Duration::from_secs(4),
		..Default::default()
	})?;
	manager.backend_mut().on_start_processing(0.0);
	for i in (1..=3).rev() {
		assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(i as f32));
		assert_eq!(instance.state(), InstanceState::Stopping);
	}
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(0.0));
	assert_eq!(instance.state(), InstanceState::Stopped);
	Ok(())
}

#[test]
#[allow(clippy::float_cmp)]
fn pauses_and_resumes_with_fade() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::new(Default::default(), MockBackend::new(1)).unwrap();
	let mut sound = manager.add_sound(StaticSound::from_frames(
		1,
		vec![Frame::from_mono(4.0); 10],
		Default::default(),
	))?;
	let mut instance = sound.play(Default::default())?;
	manager.backend_mut().on_start_processing(0.0);

	// make sure playback is happening normally
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(4.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 1.0);
	assert_eq!(instance.state(), InstanceState::Playing);

	// pause the instance
	instance.pause(Tween {
		duration: Duration::from_secs(4),
		..Default::default()
	})?;
	manager.backend_mut().on_start_processing(0.0);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(3.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 2.0);
	assert_eq!(instance.state(), InstanceState::Pausing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(2.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 3.0);
	assert_eq!(instance.state(), InstanceState::Pausing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(1.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 4.0);
	assert_eq!(instance.state(), InstanceState::Pausing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(0.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 5.0);
	assert_eq!(instance.state(), InstanceState::Paused);

	// make sure the instance position doesn't change
	for _ in 0..3 {
		assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(0.0));
		manager.backend_mut().on_start_processing(0.0);
		assert_eq!(instance.position(), 5.0);
		assert_eq!(instance.state(), InstanceState::Paused);
	}

	// resume the instance
	instance.resume(Tween {
		duration: Duration::from_secs(4),
		..Default::default()
	})?;
	manager.backend_mut().on_start_processing(0.0);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(1.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 6.0);
	assert_eq!(instance.state(), InstanceState::Playing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(2.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 7.0);
	assert_eq!(instance.state(), InstanceState::Playing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(3.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 8.0);
	assert_eq!(instance.state(), InstanceState::Playing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(4.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 9.0);
	assert_eq!(instance.state(), InstanceState::Playing);
	assert_frame_approximate_eq(manager.backend_mut().process(), Frame::from_mono(4.0));
	manager.backend_mut().on_start_processing(0.0);
	assert_eq!(instance.position(), 10.0);
	assert_eq!(instance.state(), InstanceState::Playing);

	Ok(())
}
