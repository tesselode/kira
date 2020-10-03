mod duration;
mod event;
mod instance;
mod manager;
mod metronome;
mod sequence;
mod sound;
mod tween;

use event::LCustomEventHandle;
use manager::{LAudioManager, LAudioManagerSettings};
use mlua::prelude::*;
use mlua_derive::lua_module;
use sequence::LSequence;

fn new_manager(_: &Lua, settings: LAudioManagerSettings) -> LuaResult<LAudioManager> {
	Ok(LAudioManager::new(settings).unwrap())
}

fn new_sequence(_: &Lua, _: ()) -> LuaResult<LSequence> {
	Ok(LSequence::new())
}

fn new_custom_event_handle(_: &Lua, _: ()) -> LuaResult<LCustomEventHandle> {
	Ok(LCustomEventHandle::new())
}

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("newManager", lua.create_function(new_manager)?)?;
	exports.set("newSequence", lua.create_function(new_sequence)?)?;
	exports.set(
		"newCustomEventHandle",
		lua.create_function(new_custom_event_handle)?,
	)?;
	Ok(exports)
}
