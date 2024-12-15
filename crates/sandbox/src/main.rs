use std::{error::Error, io::stdin, time::Duration};

use kira::{
	backend::DefaultBackend,
	clock::ClockSpeed,
	effect::panning_control::PanningControlBuilder,
	manager::{AudioManager, AudioManagerSettings},
	modulator::tweener::TweenerBuilder,
	sound::sine::SineBuilder,
	track::MainTrackBuilder,
	Decibels, Easing, Mapping, Panning, StartTime, Tween, Value,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings {
		main_track_builder: MainTrackBuilder::new()
			.with_effect(PanningControlBuilder(Value::Fixed(Panning::LEFT))),
		..Default::default()
	})?;
	let mut tweener = manager.add_modulator(TweenerBuilder { initial_value: 0.0 })?;
	let mut clock = manager.add_clock(Value::FromModulator {
		id: tweener.id(),
		mapping: Mapping {
			input_range: (0.0, 1.0),
			output_range: (
				ClockSpeed::TicksPerMinute(120.0),
				ClockSpeed::TicksPerMinute(240.0),
			),
			easing: Easing::Linear,
		},
	})?;
	tweener.set(
		1.0,
		Tween {
			duration: Duration::from_secs(5),
			start_time: StartTime::ClockTime(clock.time() + 4),
			..Default::default()
		},
	);
	for i in 0..16 {
		manager.play(SineBuilder {
			frequency: Value::Fixed(100.0 + 100.0 * i as f64),
			start_time: StartTime::ClockTime(clock.time() + i),
		})?;
	}
	clock.start();
	manager.main_track().set_volume(
		Decibels::SILENCE,
		Tween {
			duration: Duration::from_secs(5),
			start_time: StartTime::ClockTime(clock.time() + 4),
			..Default::default()
		},
	);

	loop {
		println!("{:?}", clock.time());
		std::thread::sleep(Duration::from_millis(50));
	}
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
