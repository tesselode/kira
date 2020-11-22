use kira::parameter::{EaseDirection, Easing, Tween};
use mlua::prelude::*;

use crate::error::KiraLuaError;

pub struct LTween(pub Tween);

impl<'lua> FromLua<'lua> for LTween {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Integer(duration) => Ok(LTween((duration as f64).into())),
			LuaValue::Number(duration) => Ok(LTween(duration.into())),
			LuaValue::Table(table) => Ok(LTween(Tween {
				duration: table.get(1)?,
				easing: Easing::default(),
				ease_direction: EaseDirection::default(),
			})),
			value => Err(LuaError::external(KiraLuaError::wrong_argument_type(
				"tween",
				"table or number",
				value,
			))),
		}
	}
}
