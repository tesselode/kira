use conductor::sound::SoundId;
use mlua::prelude::*;

#[derive(Copy, Clone)]
pub struct LSoundId(pub SoundId);

impl LuaUserData for LSoundId {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("duration", |_, this, _: ()| {
			let duration = this.0.duration();
			Ok(duration)
		})
	}
}
