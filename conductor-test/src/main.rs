use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sound::{SoundMetadata, SoundSettings},
	Tempo, Tween,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let pitch_parameter_id = manager.add_parameter(1.0)?;
	let track_id = manager.add_sub_track(Default::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir().unwrap().join("assets/loop.ogg"),
		SoundSettings {
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	manager.set_parameter(pitch_parameter_id, 0.25, Some(Tween(2.0)))?;
	manager.play_sound(
		sound_id,
		InstanceSettings::new()
			.pitch(pitch_parameter_id)
			.track(track_id)
			.loop_region(..),
	)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
