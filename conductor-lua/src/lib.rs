mod manager;

use manager::LAudioManager;
use mlua::prelude::*;
use mlua_derive::lua_module;

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let table = lua.create_table()?;
	table.set(
		"newManager",
		lua.create_function(|_: &Lua, _: ()| LAudioManager::new())?,
	)?;
	Ok(table)
}
