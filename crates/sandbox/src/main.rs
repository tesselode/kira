use std::{error::Error, io::stdin, time::Duration};

use kira::{
	backend::DefaultBackend,
	clock::ClockSpeed,
	effect::panning_control::PanningControlBuilder,
	manager::{AudioManager, AudioManagerSettings},
	modulator::tweener::TweenerBuilder,
	sound::sine::SineBuilder,
	track::{MainTrackBuilder, TrackBuilder},
	Decibels, Easing, Mapping, Panning, StartTime, Tween, Value,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let mut track_1 = manager.add_sub_track(TrackBuilder::new().volume(-6.0))?;
	track_1.play(SineBuilder {
		frequency: Value::Fixed(440.0),
		..Default::default()
	})?;
	let mut track_2 = manager.add_sub_track(TrackBuilder::new().volume(-6.0))?;
	track_2.play(SineBuilder {
		frequency: Value::Fixed(660.0),
		..Default::default()
	})?;

	wait_for_enter_press()?;

	track_1.pause(Tween {
		duration: Duration::from_secs(1),
		..Default::default()
	});

	wait_for_enter_press()?;

	track_1.resume(Tween {
		duration: Duration::from_secs(1),
		..Default::default()
	});

	wait_for_enter_press()?;

	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
