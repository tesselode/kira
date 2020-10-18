use conductor::manager::{AudioManager, AudioManagerSettings};
use mlua::prelude::*;

pub struct LAudioManager(pub AudioManager);

impl LAudioManager {
	pub fn new() -> LuaResult<Self> {
		match AudioManager::new(AudioManagerSettings::default()) {
			Ok(manager) => Ok(Self(manager)),
			Err(error) => Err(LuaError::external(error)),
		}
	}
}

impl LuaUserData for LAudioManager {}
