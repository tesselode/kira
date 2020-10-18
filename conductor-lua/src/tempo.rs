use conductor::tempo::Tempo;
use mlua::prelude::*;

use crate::error::ConductorLuaError;

pub struct LTempo(pub Tempo);

impl<'lua> FromLua<'lua> for LTempo {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Number(bpm) => Ok(LTempo(Tempo(bpm))),
			_ => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"tempo", "number",
			))),
		}
	}
}

impl<'lua> ToLua<'lua> for LTempo {
	fn to_lua(self, _: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		Ok(LuaValue::Number((self.0).0))
	}
}
