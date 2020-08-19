use crate::{
	instance::{LInstanceId, LInstanceSettings},
	sequence::{LSequence, LSequenceId},
	sound::{LSoundId, LSoundMetadata},
	tween::LTween,
};
use conductor::{
	instance::InstanceSettings,
	manager::{AudioManager, AudioManagerSettings},
	sound::SoundMetadata,
};
use mlua::prelude::*;
use std::{error::Error, path::PathBuf};

pub struct LAudioManager(AudioManager);

impl LAudioManager {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		Ok(Self(AudioManager::new(AudioManagerSettings::default())?))
	}
}

impl LuaUserData for LAudioManager {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut(
			"loadSound",
			|_, this, (path, metadata): (LuaString, LSoundMetadata)| {
				let path = std::env::current_dir()
					.unwrap()
					.join(PathBuf::from(path.to_str()?));
				let sound_id = this.0.load_sound(&path, metadata.0).unwrap();
				Ok(LSoundId(sound_id))
			},
		);

		methods.add_method_mut(
			"playSound",
			|_, this, (id, settings): (LSoundId, LInstanceSettings)| {
				let instance_id = this.0.play_sound(id.0, settings.0).unwrap();
				Ok(LInstanceId(instance_id))
			},
		);

		methods.add_method_mut(
			"setInstanceVolume",
			|_, this, (id, volume, tween): (LInstanceId, f32, Option<LTween>)| {
				this.0
					.set_instance_volume(
						id.0,
						volume,
						match tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut(
			"setInstancePitch",
			|_, this, (id, pitch, tween): (LInstanceId, f32, Option<LTween>)| {
				this.0
					.set_instance_pitch(
						id.0,
						pitch,
						match tween {
							Some(tween) => Some(tween.0),
							None => None,
						},
					)
					.unwrap();
				Ok(())
			},
		);

		methods.add_method_mut("startSequence", |_, this, sequence: LSequence| {
			let id = this.0.start_sequence(sequence.0).unwrap();
			Ok(LSequenceId(id))
		});
	}
}
