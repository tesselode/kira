use conductor::tween::Tween;
use mlua::prelude::*;

pub struct LTween(pub Tween);

impl LuaUserData for LTween {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut("setDuration", |_, this, duration: f64| {
			(this.0).0 = duration;
			Ok(())
		});
	}
}

impl<'lua> FromLua<'lua> for LTween {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Table(table) => Ok(LTween(Tween(table.get("duration")?))),
			_ => Err(LuaError::FromLuaConversionError {
				from: "table",
				to: "Tween",
				message: None,
			}),
		}
	}
}
