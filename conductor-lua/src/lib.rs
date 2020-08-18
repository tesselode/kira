mod instance;
mod manager;
mod sound;

use manager::LAudioManager;
use mlua::prelude::*;
use mlua_derive::lua_module;

fn new_manager(_: &Lua, _: ()) -> LuaResult<LAudioManager> {
	Ok(LAudioManager::new().unwrap())
}

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("new_manager", lua.create_function(new_manager)?)?;
	Ok(exports)
}
