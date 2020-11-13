use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sequence::Sequence,
	sound::{SoundMetadata, SoundSettings},
	Duration, Tempo,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir().unwrap().join("assets/loop.ogg"),
		SoundSettings {
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	let mut sequence = Sequence::new();
	sequence.wait_for_interval(1.0);
	sequence.start_loop();
	let instance_id = sequence.play_sound(sound_id, Default::default());
	sequence.wait(Duration::Beats(1.0));
	sequence.stop_instance(instance_id, None);
	sequence.wait(Duration::Beats(1.0));
	let instance_id = sequence.play_sound(
		sound_id,
		InstanceSettings {
			pitch: 2.0.into(),
			..Default::default()
		},
	);
	sequence.wait(Duration::Beats(1.0));
	sequence.stop_instance(instance_id, None);
	sequence.wait(Duration::Beats(1.0));
	manager.start_sequence(sequence)?;
	manager.set_metronome_tempo(128.0.into())?;
	manager.start_metronome()?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
