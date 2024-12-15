use std::{error::Error, io::stdin};

use kira::{
	backend::DefaultBackend,
	effect::reverb::ReverbBuilder,
	manager::{AudioManager, AudioManagerSettings},
	modulator::lfo::LfoBuilder,
	sound::sine::SineBuilder,
	track::{SendTrackBuilder, TrackBuilder},
	Easing, Mapping, Mix, Value,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let reverb_send = manager
		.add_send_track(SendTrackBuilder::new().with_effect(ReverbBuilder::new().mix(Mix::WET)))?;
	let mut track = manager.add_sub_track(TrackBuilder::new().with_send(&reverb_send, -6.0))?;
	let lfo = manager.add_modulator(LfoBuilder::new())?;
	track.play(SineBuilder {
		frequency: Value::FromModulator {
			id: lfo.id(),
			mapping: Mapping {
				input_range: (-1.0, 1.0),
				output_range: (220.0, 440.0),
				easing: Easing::Linear,
			},
		},
		..Default::default()
	})?;

	wait_for_enter_press()?;

	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
