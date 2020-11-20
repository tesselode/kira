use kira::parameter::{Mapping, ParameterId};
use mlua::prelude::*;

use crate::error::KiraLuaError;

#[derive(Debug, Copy, Clone)]
pub struct LMapping(pub Mapping);

impl<'lua> FromLua<'lua> for LMapping {
	fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LMapping(Mapping::default())),
			LuaValue::Table(table) => {
				let mut mapping = Mapping::default();
				if table.contains_key("inputRange")? {
					let input_range_table = table.get::<_, LuaTable>("inputRange")?;
					mapping.input_range = (
						input_range_table.get::<_, f64>(1)?,
						input_range_table.get::<_, f64>(2)?,
					)
				}
				if table.contains_key("outputRange")? {
					let output_range_table = table.get::<_, LuaTable>("outputRange")?;
					mapping.output_range = (
						output_range_table.get::<_, f64>(1)?,
						output_range_table.get::<_, f64>(2)?,
					)
				}
				if table.contains_key("clampBottom")? {
					mapping.clamp_bottom = table.get::<_, bool>("clampBottom")?;
				}
				if table.contains_key("clampTop")? {
					mapping.clamp_top = table.get::<_, bool>("clampTop")?;
				}
				Ok(LMapping(mapping))
			}
			value => Err(LuaError::external(KiraLuaError::wrong_argument_type(
				"mapping", "table", value,
			))),
		}
	}
}

#[derive(Debug, Clone)]
pub struct LParameterId(pub ParameterId);

impl LuaUserData for LParameterId {}
