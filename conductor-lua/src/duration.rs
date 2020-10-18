use conductor::duration::Duration;
use mlua::prelude::*;

pub enum DurationUnit {
	Seconds,
	Beats,
}

impl<'lua> FromLua<'lua> for DurationUnit {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::String(string) => match string.to_str()? {
				"second" => Ok(DurationUnit::Seconds),
				"seconds" => Ok(DurationUnit::Seconds),
				"beat" => Ok(DurationUnit::Beats),
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
	pub fn new(value: f64, unit: DurationUnit) -> Self {
		Self(match unit {
			DurationUnit::Seconds => Duration::Seconds(value),
			DurationUnit::Beats => Duration::Beats(value),
		})
	}
}

impl<'lua> FromLua<'lua> for LDuration {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Table(table) => Ok(LDuration::new(table.get(1)?, table.get(2)?)),
			_ => panic!(),
		}
	}
}

impl<'lua> ToLua<'lua> for LDuration {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		Ok(LuaValue::Table(match self.0 {
			Duration::Seconds(seconds) => {
				let table = lua.create_table()?;
				table.set(1, seconds)?;
				table.set(2, lua.create_string("seconds")?)?;
				table
			}
			Duration::Beats(beats) => {
				let table = lua.create_table()?;
				table.set(1, beats)?;
				table.set(2, lua.create_string("beats")?)?;
				table
			}
		}))
	}
}
