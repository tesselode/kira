mod duration;
mod instance;
mod manager;
mod sequence;
mod sound;

use manager::LAudioManager;
use mlua::prelude::*;
use mlua_derive::lua_module;
use sequence::LSequence;

fn new_manager(_: &Lua, _: ()) -> LuaResult<LAudioManager> {
	Ok(LAudioManager::new().unwrap())
}

fn new_sequence(_: &Lua, _: ()) -> LuaResult<LSequence> {
	Ok(LSequence::new())
}

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("newManager", lua.create_function(new_manager)?)?;
	exports.set("newSequence", lua.create_function(new_sequence)?)?;
	Ok(exports)
}
