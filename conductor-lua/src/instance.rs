use conductor::instance::{InstanceId, InstanceSettings};
use mlua::prelude::*;

use crate::error::ConductorLuaError;

pub struct LInstanceSettings(pub InstanceSettings);

impl<'lua> FromLua<'lua> for LInstanceSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LInstanceSettings(InstanceSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = InstanceSettings::default();
				if table.contains_key("volume")? {
					settings.volume = table.get("volume")?;
				}
				if table.contains_key("pitch")? {
					settings.pitch = table.get("pitch")?;
				}
				if table.contains_key("position")? {
					settings.position = table.get("position")?;
				}
				if table.contains_key("fadeInDuration")? {
					settings.fade_in_duration = table.get("fadeInDuration")?;
				}
				Ok(LInstanceSettings(settings))
			}
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"instance settings",
				"table",
				value,
			))),
		}
	}
}

#[derive(Debug, Clone)]
pub struct LInstanceId(pub InstanceId);

impl LuaUserData for LInstanceId {}
