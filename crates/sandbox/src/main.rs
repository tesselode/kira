use std::{error::Error, f32::consts::TAU, io::stdin};

use kira::{
	backend::DefaultBackend,
	manager::{AudioManager, AudioManagerSettings},
	sound::{Sound, SoundData},
	Frame,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	manager.play(Sine::new(440.0))?;
	wait_for_enter_press()?;
	manager.play(Sine::new(220.0))?;
	wait_for_enter_press()?;
	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}

pub struct Sine {
	frequency: f32,
	phase: f32,
}

impl Sine {
	pub fn new(frequency: f32) -> Self {
		Self {
			frequency,
			phase: 0.0,
		}
	}
}

impl SoundData for Sine {
	type Error = ();

	type Handle = ();

	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		Ok((Box::new(self), ()))
	}
}

impl Sound for Sine {
	fn process(&mut self, out: &mut [Frame], dt: f64) {
		for frame in out {
			*frame = Frame::from_mono(0.1 * (self.phase * TAU).sin());
			self.phase += self.frequency * dt as f32;
			self.phase %= 1.0;
		}
	}

	fn finished(&self) -> bool {
		false
	}
}
