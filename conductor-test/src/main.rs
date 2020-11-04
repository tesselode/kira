use std::{error::Error, io::stdin};

use conductor::{
	instance::{InstanceSettings, LoopRegion},
	manager::{AudioManager, AudioManagerSettings},
	sound::{SoundMetadata, SoundSettings},
	tempo::Tempo,
	track::effect::svf::StateVariableFilter,
	track::effect::svf::StateVariableFilterSettings,
	track::TrackSettings,
	track::{effect::svf::StateVariableFilterMode, index::TrackIndex, EffectSettings},
	tween::Tween,
	value::Value,
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
	manager.play_sound(sound_id, InstanceSettings::default().loop_region(..))?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
