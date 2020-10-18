use conductor::duration::Duration;
use mlua::prelude::*;

use crate::error::ConductorLuaError;

pub struct LDuration(pub Duration);

impl<'lua> FromLua<'lua> for LDuration {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Table(table) => {
				let units_lua_string = table.get::<_, LuaString>(2)?;
				let units_str = units_lua_string.to_str()?;
				if units_str == "second" || units_str == "seconds" {
					Ok(LDuration(Duration::Seconds(table.get(1)?)))
				} else if units_str == "beat" || units_str == "beats" {
					Ok(LDuration(Duration::Beats(table.get(1)?)))
				} else {
					return Err(LuaError::external(ConductorLuaError::InvalidDurationUnit));
				}
			}
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"duration", "table", value,
			))),
		}
	}
}

impl<'lua> ToLua<'lua> for LDuration {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		let table = lua.create_table()?;
		match self.0 {
			Duration::Seconds(seconds) => {
				table.set(1, seconds)?;
				table.set(2, lua.create_string("seconds")?)?;
			}
			Duration::Beats(beats) => {
				table.set(1, beats)?;
				table.set(2, lua.create_string("beats")?)?;
			}
		}
		Ok(LuaValue::Table(table))
	}
}
