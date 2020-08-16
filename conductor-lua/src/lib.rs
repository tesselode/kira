use mlua::prelude::*;
use mlua_derive::lua_module;

fn hello(_: &Lua, _: ()) -> LuaResult<()> {
	println!("hi!");
	Ok(())
}

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("hello", lua.create_function(hello)?)?;
	Ok(exports)
}
