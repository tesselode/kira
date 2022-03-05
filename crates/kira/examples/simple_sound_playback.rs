use std::{error::Error, io::stdin};

use kira::{
	manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())?;
	let sound = StaticSoundData::from_file(
		"crates/kira/examples/blip.ogg",
		StaticSoundSettings::default(),
	)?;

	println!("Press enter to play a sound");
	loop {
		stdin().read_line(&mut "".into())?;
		manager.play(sound.clone())?;
	}
}
