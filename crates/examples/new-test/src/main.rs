use std::{error::Error, io::stdin};

use kira::{
	backend::DefaultBackend,
	manager::{AudioManager, AudioManagerSettings},
	sound::static_sound::StaticSoundData,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let sound_data = StaticSoundData::from_file("crates/examples/assets/blip.ogg")?.loop_region(..);
	manager.play(sound_data)?;

	wait_for_enter_press()?;

	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
