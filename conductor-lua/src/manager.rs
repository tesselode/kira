use conductor::manager::{AudioManager, AudioManagerSettings};
use mlua::prelude::*;

use crate::{
	error::ConductorLuaError, event::LEvent, metronome::LMetronomeSettings, sound::LSoundId,
	sound::LSoundSettings,
};

pub struct LAudioManagerSettings(pub AudioManagerSettings);

impl<'lua> FromLua<'lua> for LAudioManagerSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LAudioManagerSettings(AudioManagerSettings::default())),
			LuaValue::Table(table) => Ok(LAudioManagerSettings(AudioManagerSettings {
				num_commands: table
					.get::<_, Option<usize>>("numCommands")?
					.unwrap_or(AudioManagerSettings::default().num_commands),
				num_events: table
					.get::<_, Option<usize>>("numEvents")?
					.unwrap_or(AudioManagerSettings::default().num_events),
				num_sounds: table
					.get::<_, Option<usize>>("numSounds")?
					.unwrap_or(AudioManagerSettings::default().num_sounds),
				num_instances: table
					.get::<_, Option<usize>>("numInstances")?
					.unwrap_or(AudioManagerSettings::default().num_instances),
				num_sequences: table
					.get::<_, Option<usize>>("numSequences")?
					.unwrap_or(AudioManagerSettings::default().num_sequences),
				metronome_settings: table
					.get::<_, Option<LMetronomeSettings>>("metronomeSettings")?
					.map(|settings| settings.0)
					.unwrap_or_default(),
			})),
			_ => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"audio manager settings",
				"table",
			))),
		}
	}
}

pub struct LAudioManager(pub AudioManager<LEvent>);

impl LAudioManager {
	pub fn new(settings: LAudioManagerSettings) -> LuaResult<Self> {
		match AudioManager::new(settings.0) {
			Ok(manager) => Ok(Self(manager)),
			Err(error) => Err(LuaError::external(error)),
		}
	}
}

impl LuaUserData for LAudioManager {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut(
			"loadSound",
			|_: &Lua, this: &mut Self, (path, settings): (LuaString, LSoundSettings)| match this
				.0
				.load_sound(path.to_str()?, settings.0)
			{
				Ok(id) => Ok(LSoundId(id)),
				Err(error) => Err(LuaError::external(error)),
			},
		);
	}
}
