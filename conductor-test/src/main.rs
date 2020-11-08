use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sound::{SoundMetadata, SoundSettings},
	track::effect::svf::StateVariableFilter,
	track::effect::svf::StateVariableFilterMode,
	track::effect::svf::StateVariableFilterSettings,
	Tempo, Tween,
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let track_id = manager.add_sub_track(Default::default())?;
	let effect_id = manager.add_effect_to_track(
		track_id,
		Box::new(StateVariableFilter::new(StateVariableFilterSettings {
			mode: StateVariableFilterMode::Notch,
			cutoff: 1000.0.into(),
			resonance: 0.5.into(),
		})),
		Default::default(),
	)?;
	let sound_id = manager.load_sound(
		std::env::current_dir().unwrap().join("assets/loop.ogg"),
		SoundSettings {
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	manager.play_sound(
		sound_id,
		InstanceSettings::new().track(track_id).loop_region(..),
	)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
