use conductor::instance::{InstanceId, InstanceSettings};
use mlua::prelude::*;

use crate::error::ConductorLuaError;

pub struct LInstanceSettings(pub InstanceSettings);

impl<'lua> FromLua<'lua> for LInstanceSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LInstanceSettings(InstanceSettings::default())),
			LuaValue::Table(table) => Ok(LInstanceSettings(InstanceSettings {
				volume: table
					.get::<_, Option<f64>>("volume")?
					.unwrap_or(InstanceSettings::default().volume),
				pitch: table
					.get::<_, Option<f64>>("pitch")?
					.unwrap_or(InstanceSettings::default().pitch),
				position: table
					.get::<_, Option<f64>>("position")?
					.unwrap_or(InstanceSettings::default().position),
				fade_in_duration: table.get::<_, Option<f64>>("fadeInDuration")?,
			})),
			_ => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"instance settings",
				"table",
			))),
		}
	}
}

pub struct LInstanceId(pub InstanceId);

impl LuaUserData for LInstanceId {}
