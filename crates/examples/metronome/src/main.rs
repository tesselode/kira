use std::{error::Error, time::Duration};

use kira::{
	clock::ClockSpeed,
	manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	tween::Value,
	PlaybackRate, StartTime,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
	let sound = StaticSoundData::from_file(
		"crates/examples/assets/blip.ogg",
		StaticSoundSettings::default(),
	)?;
	let clock = manager.add_clock(ClockSpeed::TicksPerMinute(120.0))?;
	// queue up the first sound for clock tick 0 (will be triggered
	// as soon as the clock is started)
	manager.play({
		let mut sound = sound.clone();
		sound.settings.playback_rate = Value::Fixed(PlaybackRate::Factor(2.0));
		sound.settings.start_time = StartTime::ClockTime(clock.time());
		sound
	})?;
	// queue up the second sound for clock tick 1
	manager.play({
		let mut sound = sound.clone();
		sound.settings.start_time = StartTime::ClockTime(clock.time() + 1);
		sound
	})?;
	clock.start()?;

	// whenever the clock ticks, queue up the next sound
	let mut previous_clock_time = clock.time();
	loop {
		std::thread::sleep(Duration::from_millis(10));
		let current_clock_time = clock.time();
		if current_clock_time.ticks > previous_clock_time.ticks {
			manager.play({
				let mut sound = sound.clone();
				// every 4 beats, play the sound at a higher pitch
				if (current_clock_time.ticks + 1) % 4 == 0 {
					sound.settings.playback_rate = Value::Fixed(PlaybackRate::Factor(2.0));
				}
				sound.settings.start_time = StartTime::ClockTime(clock.time() + 1);
				sound
			})?;
			previous_clock_time = current_clock_time;
		}
	}
}
