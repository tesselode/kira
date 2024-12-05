use std::{error::Error, io::stdin};

use kira::{
	manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundData,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let sound_data = StaticSoundData::from_file("crates/examples/assets/blip.ogg")?;

	println!("Press enter to play a sound");
	loop {
		wait_for_enter_press()?;
		manager.play(sound_data.clone())?;
	}
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
