use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sound::SoundSettings,
	tween::Tween,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir()?.join("assets/test.mp3"),
		SoundSettings::default(),
	)?;
	let instance_id = manager.play_sound(sound_id, InstanceSettings::default())?;
	manager.set_instance_pitch(instance_id, -1.0, Some(Tween(10.0)))?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
