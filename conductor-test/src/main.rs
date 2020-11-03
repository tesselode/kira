use std::{error::Error, io::stdin};

use conductor::{
	instance::{InstanceSettings, LoopSettings},
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
	let filter_cutoff_parameter_id = manager.add_parameter(1000.0)?;
	let track_id = manager.add_sub_track(TrackSettings::default())?;
	let effect_id = manager.add_effect_to_track(
		TrackIndex::Sub(track_id),
		Box::new(StateVariableFilter::new(StateVariableFilterSettings {
			mode: StateVariableFilterMode::LowPass,
			cutoff: filter_cutoff_parameter_id.into(),
			..Default::default()
		})),
		EffectSettings::default(),
	)?;
	let sound_id = manager.load_sound(
		std::env::current_dir().unwrap().join("assets/loop.ogg"),
		SoundSettings {
			default_track: TrackIndex::Sub(track_id),
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	manager.play_sound(
		sound_id,
		InstanceSettings {
			loop_settings: Some(LoopSettings::default()),
			..Default::default()
		},
	)?;
	manager.set_parameter(filter_cutoff_parameter_id, 4000.0, Some(Tween(5.0)))?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	manager.remove_parameter(filter_cutoff_parameter_id)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
