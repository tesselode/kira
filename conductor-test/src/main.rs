use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sequence::Sequence,
	sequence::SoundLoopSettings,
	sound::SoundMetadata,
	sound::SoundSettings,
	tempo::Tempo,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir()?.join("assets/loop.ogg"),
		SoundSettings {
			metadata: SoundMetadata {
				tempo: Some(Tempo(128.0)),
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	let sequence = Sequence::new_sound_loop(
		sound_id,
		SoundLoopSettings {
			start_point: Some(sound_id.metadata().tempo.unwrap().beats_to_seconds(8.0)),
			end_point: None,
		},
		InstanceSettings {
			//position: sound_id.metadata().tempo.unwrap().beats_to_seconds(15.0),
			..Default::default()
		},
	);
	manager.start_sequence(sequence)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
