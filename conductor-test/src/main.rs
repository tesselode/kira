use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sound::SoundSettings,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir()?.join("assets/test.wav"),
		SoundSettings::default(),
	)?;
	manager.play_sound(sound_id, InstanceSettings::default())?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
