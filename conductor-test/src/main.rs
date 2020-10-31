use std::{error::Error, io::stdin};

use conductor::{
	instance::{InstanceSettings, LoopSettings},
	manager::{AudioManager, AudioManagerSettings},
	sound::{SoundMetadata, SoundSettings},
	tempo::Tempo,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir()?.join("assets/loop.ogg"),
		SoundSettings {
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	manager.play_sound(
		sound_id,
		InstanceSettings {
			pitch: 4.0,
			loop_settings: Some(LoopSettings::default()),
			..Default::default()
		},
	)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
