use conductor::duration::Duration;
use mlua::prelude::*;

pub enum DurationUnit {
	Seconds,
	Beats,
}

impl<'lua> FromLua<'lua> for DurationUnit {
	fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::String(string) => match string.to_str()? {
				"seconds" => Ok(DurationUnit::Seconds),
				"beats" => Ok(DurationUnit::Beats),
				_ => Err(LuaError::FromLuaConversionError {
					from: "string",
					to: "DurationUnit",
					message: None,
				}),
			},
			_ => Err(LuaError::FromLuaConversionError {
				from: "number",
				to: "DurationUnit",
				message: None,
			}),
		}
	}
}

pub struct LDuration(pub Duration);

impl LDuration {
	pub fn new(value: f32, unit: DurationUnit) -> Self {
		Self(match unit {
			DurationUnit::Seconds => Duration::Seconds(value),
			DurationUnit::Beats => Duration::Beats(value),
		})
	}
}

impl LuaUserData for LDuration {}
