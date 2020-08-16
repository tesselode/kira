use conductor::manager::{AudioManager, AudioManagerSettings};
use mlua::prelude::*;
use mlua_derive::lua_module;

fn new_manager(_: &Lua, _: ()) -> LuaResult<()> {
	let audio_manager = AudioManager::new(AudioManagerSettings::default()).unwrap();
	Ok(())
}

#[lua_module]
fn conductor(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("new_manager", lua.create_function(new_manager)?)?;
	Ok(exports)
}
