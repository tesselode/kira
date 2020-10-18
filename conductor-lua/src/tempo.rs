use conductor::tempo::Tempo;
use mlua::prelude::*;

use crate::error::LConductorError;

pub struct LTempo(pub Tempo);

impl<'lua> FromLua<'lua> for LTempo {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Number(bpm) => Ok(LTempo(Tempo(bpm))),
			_ => Err(LConductorError::wrong_argument_type("tempo", "number")),
		}
	}
}
