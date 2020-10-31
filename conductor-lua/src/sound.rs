use conductor::sound::{SoundId, SoundMetadata, SoundSettings};
use mlua::prelude::*;

use crate::error::ConductorLuaError;

pub struct LSoundMetadata(SoundMetadata);

impl<'lua> FromLua<'lua> for LSoundMetadata {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LSoundMetadata(SoundMetadata::default())),
			LuaValue::Table(table) => {
				let mut metadata = SoundMetadata::default();
				if table.contains_key("semanticDuration")? {
					metadata.semantic_duration = Some(table.get("semanticDuration")?);
				}
				Ok(LSoundMetadata(metadata))
			}
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"sound metadata",
				"table",
				value,
			))),
		}
	}
}

impl<'lua> ToLua<'lua> for LSoundMetadata {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		let table = lua.create_table()?;
		table.set("semanticDuration", self.0.semantic_duration)?;
		Ok(LuaValue::Table(table))
	}
}

pub struct LSoundSettings(pub SoundSettings);

impl<'lua> FromLua<'lua> for LSoundSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LSoundSettings(SoundSettings::default())),
			LuaValue::Table(table) => Ok(LSoundSettings(SoundSettings {
				cooldown: Some(table.get::<_, Option<f64>>("cooldown")?.unwrap_or(0.0001)),
				metadata: table
					.get::<_, Option<LSoundMetadata>>("metadata")?
					.map(|metadata| metadata.0)
					.unwrap_or_default(),
			})),
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"sound settings",
				"table",
				value,
			))),
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct LSoundId(pub SoundId);

impl LuaUserData for LSoundId {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("getDuration", |_: &Lua, this: &Self, _: ()| {
			Ok(this.0.duration())
		});

		methods.add_method("getMetadata", |_: &Lua, this: &Self, _: ()| {
			Ok(LSoundMetadata(*this.0.metadata()))
		})
	}
}
