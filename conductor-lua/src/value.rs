use conductor::Value;
use mlua::prelude::*;

use crate::{error::ConductorLuaError, parameter::LParameterId};

pub struct LValue(pub Value);

impl<'lua> FromLua<'lua> for LValue {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match &lua_value {
			LuaValue::Number(value) => return Ok(LValue(Value::Fixed(*value))),
			LuaValue::UserData(user_data) => {
				if user_data.is::<LParameterId>() {
					let id = user_data.borrow::<LParameterId>()?;
					return Ok(LValue(Value::Parameter(id.0)));
				}
			}
			_ => {}
		}
		Err(LuaError::external(ConductorLuaError::wrong_argument_type(
			"value",
			"number or parameter",
			lua_value,
		)))
	}
}
