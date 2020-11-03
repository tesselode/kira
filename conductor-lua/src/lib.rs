mod duration;
mod error;
mod event;
mod instance;
mod manager;
mod metronome;
mod parameter;
mod sequence;
mod sound;
mod tempo;
mod track;
mod tween;
mod value;

use conductor::sequence::Sequence;
use event::CustomEvent;
use manager::{LAudioManager, LAudioManagerSettings};
use mlua::prelude::*;
use mlua_derive::lua_module;
use sequence::LSequence;

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
		"newSequence",
		lua.create_function(|_: &Lua, _: ()| Ok(LSequence(Sequence::new())))?,
	)?;
	table.set(
		"newCustomEvent",
		lua.create_function(|_: &Lua, _: ()| Ok(CustomEvent::new()))?,
	)?;
	Ok(table)
}
