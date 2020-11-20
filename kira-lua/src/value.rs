use kira::{parameter::Mapping, Value};
use mlua::prelude::*;

use crate::{
	error::KiraLuaError,
	parameter::{LMapping, LParameterId},
};

pub struct LValue<T: From<f64> + Copy>(pub Value<T>);

impl<'lua, T: From<f64> + Copy> FromLua<'lua> for LValue<T> {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match &lua_value {
			LuaValue::Number(value) => return Ok(LValue(Value::Fixed(T::from(*value)))),
			LuaValue::Table(table) => {
				return Ok(LValue(Value::Parameter(
					table.get::<_, LParameterId>(1)?.0,
					table.get::<_, LMapping>(2)?.0,
				)))
			}
			LuaValue::UserData(user_data) => {
				if user_data.is::<LParameterId>() {
					let id = user_data.borrow::<LParameterId>()?;
					return Ok(LValue(Value::Parameter(id.0, Mapping::default())));
				}
			}
			_ => {}
		}
		Err(LuaError::external(KiraLuaError::wrong_argument_type(
			"value",
			"number, parameter, or table",
			lua_value,
		)))
	}
}
