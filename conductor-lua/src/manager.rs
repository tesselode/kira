use conductor::manager::{AudioManager, AudioManagerSettings};
use mlua::prelude::*;

use crate::{
	error::ConductorLuaError, event::LEvent, instance::LInstanceId, instance::LInstanceSettings,
	metronome::LMetronomeSettings, sound::LSoundId, sound::LSoundSettings,
};

pub struct LAudioManagerSettings(pub AudioManagerSettings);

impl<'lua> FromLua<'lua> for LAudioManagerSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LAudioManagerSettings(AudioManagerSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = AudioManagerSettings::default();
				if table.contains_key("numCommands")? {
					settings.num_commands = table.get("numCommands")?;
				}
				if table.contains_key("numEvents")? {
					settings.num_events = table.get("numEvents")?;
				}
				if table.contains_key("numSounds")? {
					settings.num_sounds = table.get("numSounds")?;
				}
				if table.contains_key("numInstances")? {
					settings.num_instances = table.get("numInstances")?;
				}
				if table.contains_key("numSequences")? {
					settings.num_sequences = table.get("numSequences")?;
				}
				if table.contains_key("metronomeSettings")? {
					settings.metronome_settings =
						table.get::<_, LMetronomeSettings>("metronomeSettings")?.0;
				}
				Ok(LAudioManagerSettings(settings))
			}
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

		methods.add_method_mut(
			"playSound",
			|_: &Lua, this: &mut Self, (sound_id, settings): (LSoundId, LInstanceSettings)| {
				match this.0.play_sound(sound_id.0, settings.0) {
					Ok(id) => Ok(LInstanceId(id)),
					Err(error) => Err(LuaError::external(error)),
				}
			},
		);
	}
}
