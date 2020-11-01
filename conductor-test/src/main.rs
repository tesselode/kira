use std::{error::Error, io::stdin};

use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sound::{SoundMetadata, SoundSettings},
	tempo::Tempo,
	track::effect::svf::StateVariableFilter,
	track::effect::svf::StateVariableFilterSettings,
	track::TrackSettings,
	track::{effect::svf::StateVariableFilterMode, index::TrackIndex, EffectSettings},
};

fn main() -> Result<(), Box<dyn Error>> {
	let mut manager = AudioManager::<()>::new(AudioManagerSettings::default())?;
	let sub_track_1 = manager.add_sub_track(TrackSettings::default())?;
	manager.add_effect_to_track(
		TrackIndex::Sub(sub_track_1),
		Box::new(StateVariableFilter::new(StateVariableFilterSettings {
			mode: StateVariableFilterMode::LowPass,
			cutoff: 4000.0,
			resonance: 1.0,
			..Default::default()
		})),
		EffectSettings::default(),
	)?;
	let sub_track_2 = manager.add_sub_track(TrackSettings::default())?;
	let sound_id = manager.load_sound(
		std::env::current_dir()?.join("assets/loop.ogg"),
		SoundSettings {
			default_track: TrackIndex::Sub(sub_track_1),
			metadata: SoundMetadata {
				semantic_duration: Some(Tempo(128.0).beats_to_seconds(16.0)),
			},
			..Default::default()
		},
	)?;
	manager.play_sound(sound_id, InstanceSettings::default())?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	manager.play_sound(
		sound_id,
		InstanceSettings {
			track: Some(TrackIndex::Sub(sub_track_2)),
			..Default::default()
		},
	)?;
	let mut input = String::new();
	stdin().read_line(&mut input)?;
	Ok(())
}
