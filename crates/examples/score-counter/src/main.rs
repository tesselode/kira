use std::{error::Error, io::stdin};

use kira::{
	backend::DefaultBackend, sound::static_sound::StaticSoundData, AudioManager,
	AudioManagerSettings,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let sound_data = StaticSoundData::from_file("crates/examples/assets/score.ogg")?
		.playback_rate(1.5)
		.loop_region(..0.06);

	loop {
		println!("Press enter to start a looping score counter sound");
		wait_for_enter_press()?;
		let mut sound = manager.play(sound_data.clone())?;

		println!("Press enter to finish the score counter");
		wait_for_enter_press()?;
		sound.set_loop_region(None);
	}
}

fn wait_for_enter_press() -> Result<(), Box<dyn Error>> {
	stdin().read_line(&mut "".into())?;
	Ok(())
}
