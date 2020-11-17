use kira::Value;
use mlua::prelude::*;

use crate::{error::KiraLuaError, parameter::LParameterId};

pub struct LValue<T: From<f64> + Copy>(pub Value<T>);

impl<'lua, T: From<f64> + Copy> FromLua<'lua> for LValue<T> {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match &lua_value {
			LuaValue::Number(value) => return Ok(LValue(Value::Fixed(T::from(*value)))),
			LuaValue::UserData(user_data) => {
				if user_data.is::<LParameterId>() {
					let id = user_data.borrow::<LParameterId>()?;
					return Ok(LValue(Value::Parameter(id.0)));
				}
			}
			_ => {}
		}
		Err(LuaError::external(KiraLuaError::wrong_argument_type(
			"value",
			"number or parameter",
			lua_value,
		)))
	}
}
