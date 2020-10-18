use std::{error::Error, io::stdin};

use conductor::{
	duration::Duration,
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings, LoopSettings},
	sequence::Sequence,
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
				semantic_duration: Some(Duration::Beats(16.0)),
			},
			..Default::default()
		},
	)?;
	let mut sequence = Sequence::new();
	sequence.wait(Duration::Seconds(1.0));
	sequence.start_loop();
	sequence.wait(Duration::Seconds(1.0));
	manager.start_sequence(sequence)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
