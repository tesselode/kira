use conductor::manager::Event;
use mlua::prelude::*;

pub struct LEvent(pub Event);

impl<'lua> ToLua<'lua> for LEvent {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		match self.0 {
			Event::MetronomeIntervalPassed(interval) => {
				let table = lua.create_table()?;
				table.set("event", "metronomeIntervalPassed")?;
				table.set("interval", interval)?;
				Ok(LuaValue::Table(table))
			}
		}
	}
}
