mod duration;
mod error;
mod event;
mod instance;
mod manager;
mod metronome;
mod sound;
mod tempo;

use event::LEvent;
use manager::{LAudioManager, LAudioManagerSettings};
use mlua::prelude::*;
use mlua_derive::lua_module;

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let table = lua.create_table()?;
	table.set(
		"newManager",
		lua.create_function(|_: &Lua, settings: LAudioManagerSettings| {
			LAudioManager::new(settings)
		})?,
	)?;
	table.set(
		"newCustomEvent",
		lua.create_function(|_: &Lua, _: ()| Ok(LEvent::new()))?,
	)?;
	Ok(table)
}
