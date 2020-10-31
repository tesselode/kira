use conductor::instance::{InstanceId, InstanceSettings, LoopSettings};
use mlua::prelude::*;

use crate::error::ConductorLuaError;

pub struct LLoopSettings(pub LoopSettings);

impl<'lua> FromLua<'lua> for LLoopSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LLoopSettings(LoopSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = LoopSettings::default();
				if table.contains_key("startPoint")? {
					settings.start = Some(table.get("startPoint")?);
				}
				if table.contains_key("endPoint")? {
					settings.end = Some(table.get("endPoint")?);
				}
				Ok(LLoopSettings(settings))
			}
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"sequence", "table", value,
			))),
		}
	}
}

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
				if table.contains_key("reverse")? {
					settings.reverse = table.get("reverse")?;
				}
				if table.contains_key("position")? {
					settings.position = table.get("position")?;
				}
				if table.contains_key("fadeInDuration")? {
					settings.fade_in_duration = table.get("fadeInDuration")?;
				}
				if table.contains_key("loopSettings")? {
					settings.loop_settings = Some(table.get::<_, LLoopSettings>("loopSettings")?.0);
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
