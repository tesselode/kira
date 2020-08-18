mod duration;
mod instance;
mod manager;
mod sound;

use duration::{DurationUnit, LDuration};
use manager::LAudioManager;
use mlua::prelude::*;
use mlua_derive::lua_module;

fn new_manager(_: &Lua, _: ()) -> LuaResult<LAudioManager> {
	Ok(LAudioManager::new().unwrap())
}

fn duration_test(_: &Lua, (value, unit): (f32, DurationUnit)) -> LuaResult<()> {
	let duration = LDuration::new(value, unit);
	println!("{:?}", duration.0);
	Ok(())
}

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("DURATION_UNIT_SECONDS", 0)?;
	exports.set("DURATION_UNIT_BEATS", 1)?;
	exports.set("newManager", lua.create_function(new_manager)?)?;
	exports.set("test", lua.create_function(duration_test)?)?;
	Ok(exports)
}
