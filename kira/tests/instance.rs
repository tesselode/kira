use std::error::Error;

use kira::{
	manager::{AudioManager, MockBackend},
	sound::{
		instance::{InstanceSettings, InstanceState},
		static_sound::StaticSound,
	},
	Frame, LoopBehavior,
};

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
	assert_eq!(
		manager.backend_mut().process(),
		Frame::from_mono(1.0).panned(0.5).panned(0.5)
	);
	assert_eq!(
		manager.backend_mut().process(),
		Frame::from_mono(2.0).panned(0.5).panned(0.5)
	);
	assert_eq!(
		manager.backend_mut().process(),
		Frame::from_mono(3.0).panned(0.5).panned(0.5)
	);
	assert_eq!(manager.backend_mut().process(), Frame::from_mono(0.0));
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
