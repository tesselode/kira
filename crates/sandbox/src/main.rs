use std::{error::Error, io::stdin};

use kira::{
	backend::DefaultBackend,
	manager::{AudioManager, AudioManagerSettings},
};

fn main() -> Result<(), Box<dyn Error>> {
	let _manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	wait_for_enter_press()?;
	Ok(())
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
