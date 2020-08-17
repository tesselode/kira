use crate::sound::{LSoundId, SoundInfo};
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

impl<'lua> ToLua<'lua> for LAudioManager {
	fn to_lua(mut self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		let exports = lua.create_table()?;
		exports.set(
			"load_sound",
			lua.create_function_mut(move |_: &Lua, path: LuaString| {
				let path = std::env::current_dir()
					.unwrap()
					.join(PathBuf::from(path.to_str()?));
				let id = LSoundId(self.0.load_sound(&path, SoundMetadata::default()).unwrap());
				let info = SoundInfo(id);
				Ok(info)
			})?,
		)?;
		exports.set(
			"play_sound",
			lua.create_function_mut(|_: &Lua, info: SoundInfo| {
				self.0
					.play_sound((info.0).0, InstanceSettings::default())
					.unwrap();
				Ok(())
			})?,
		)?;
		Ok(LuaValue::Table(exports))
	}
}
