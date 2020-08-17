use conductor::sound::SoundId;
use mlua::prelude::*;

#[derive(Copy, Clone)]
pub struct LSoundId(pub SoundId);

impl LuaUserData for LSoundId {}

pub struct SoundInfo(pub LSoundId);

impl<'lua> ToLua<'lua> for SoundInfo {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		let exports = lua.create_table()?;
		exports.set("_id", self.0)?;
		exports.set("duration", (self.0).0.duration())?;
		Ok(LuaValue::Table(exports))
	}
}

impl<'lua> FromLua<'lua> for SoundInfo {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaValue::Table(table) => {
				let id = table.get::<&str, LSoundId>("_id")?;
				Ok(Self(id))
			}
			_ => panic!(),
		}
	}
}
