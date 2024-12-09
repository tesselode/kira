use std::{error::Error, io::stdin};

use kira_old::{
	manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundData,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	manager.play(
		StaticSoundData::from_file("crates/examples/assets/drums.ogg")?.loop_region(3.6..6.0),
	)?;

	println!("Press enter to exit");
	wait_for_enter_press()?;

	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
