use conductor::{
	sound::{SoundId, SoundMetadata, SoundSettings},
	tempo::Tempo,
};
use mlua::prelude::*;

#[derive(Copy, Clone)]
pub struct LSoundId(pub SoundId);

impl LuaUserData for LSoundId {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("getDuration", |_, this, _: ()| Ok(this.0.duration()));

		methods.add_method("getMetadata", |_, this, _: ()| {
			Ok(LSoundMetadata(*this.0.metadata()))
		});
	}
}

pub struct LSoundMetadata(pub SoundMetadata);

impl LuaUserData for LSoundMetadata {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("getTempo", |_, this, _: ()| {
			Ok(match this.0.tempo {
				Some(tempo) => LuaValue::Number(tempo.0 as f64),
				None => LuaValue::Nil,
			})
		})
	}
}

impl<'lua> FromLua<'lua> for LSoundMetadata {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LSoundMetadata(SoundMetadata::default())),
			LuaValue::Table(table) => {
				let mut metadata = SoundMetadata::default();
				if table.contains_key("tempo")? {
					metadata.tempo = Some(Tempo(table.get("tempo")?));
				}
				Ok(LSoundMetadata(metadata))
			}
			_ => Err(LuaError::FromLuaConversionError {
				from: "table",
				to: "SoundMetadata",
				message: None,
			}),
		}
	}
}

pub struct LSoundSettings(pub SoundSettings);

impl<'lua> FromLua<'lua> for LSoundSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LSoundSettings(SoundSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = SoundSettings::default();
				if table.contains_key("cooldown")? {
					settings.cooldown = Some(table.get("cooldown")?);
				}
				if table.contains_key("metadata")? {
					let l_metadata: LSoundMetadata = table.get("metadata")?;
					settings.metadata = l_metadata.0;
				}
				Ok(LSoundSettings(settings))
			}
			_ => Err(LuaError::FromLuaConversionError {
				from: "table",
				to: "SoundSettings",
				message: None,
			}),
		}
	}
}
