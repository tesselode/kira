use std::{error::Error, io::stdin};

use conductor::{
	duration::Duration,
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sequence::Sequence,
	sound::{SoundMetadata, SoundSettings},
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
	sequence.wait_for_interval(1.0);
	sequence.start_loop();
	sequence.play_sound(
		sound_id,
		InstanceSettings {
			pitch: 4.0,
			..Default::default()
		},
	);
	sequence.wait(sound_id.metadata().semantic_duration.unwrap() / 4.0);
	let sequence_id = manager.start_sequence(sequence)?;
	manager.set_metronome_tempo(sound_id.metadata().tempo.unwrap())?;
	manager.start_metronome()?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	manager.pause_sequence_and_instances(sequence_id, None)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	manager.resume_sequence_and_instances(sequence_id, None)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
