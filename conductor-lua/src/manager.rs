use crate::sound::LSoundId;
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
		methods.add_method_mut("load_sound", |_, this, path: LuaString| {
			let path = std::env::current_dir()
				.unwrap()
				.join(PathBuf::from(path.to_str()?));
			let id = this.0.load_sound(&path, SoundMetadata::default()).unwrap();
			Ok(LSoundId(id))
		});
	}
}
