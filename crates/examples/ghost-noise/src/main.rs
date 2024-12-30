use std::{error::Error, io::stdin};

use kira::{
	backend::DefaultBackend,
	AudioManager, AudioManagerSettings,
	modulator::lfo::LfoBuilder,
	sound::static_sound::StaticSoundData,
	tween::Easing,
	Mapping, Semitones, Value,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let amplitude_lfo = manager.add_modulator(LfoBuilder::new().frequency(0.093))?;
	let frequency_lfo = manager.add_modulator(LfoBuilder::new().frequency(0.038))?;
	let playback_rate_lfo = manager.add_modulator(
		LfoBuilder::new()
			.amplitude(Value::from_modulator(
				&amplitude_lfo,
				Mapping {
					input_range: (-1.0, 1.0),
					output_range: (0.5, 1.5),
					easing: Easing::Linear,
				},
			))
			.frequency(Value::from_modulator(
				&frequency_lfo,
				Mapping {
					input_range: (-1.0, 1.0),
					output_range: (1.0, 4.0),
					easing: Easing::Linear,
				},
			)),
	)?;
	manager.play(
		StaticSoundData::from_file("crates/examples/assets/sine.wav")?
			.volume(1.0 / 3.0)
			.loop_region(..)
			.playback_rate(Value::from_modulator(
				&playback_rate_lfo,
				Mapping {
					input_range: (-1.0, 1.0),
					output_range: (Semitones(56.0).into(), Semitones(64.0).into()),
					easing: Easing::Linear,
				},
			)),
	)?;

	println!("oooOOOOooOOOOooo");
	wait_for_enter_press()?;
	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
