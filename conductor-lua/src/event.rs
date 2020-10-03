use mlua::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_CUSTOM_EVENT_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone)]
pub struct LCustomEventHandle(pub usize);

impl LCustomEventHandle {
	pub fn new() -> Self {
		let index = NEXT_CUSTOM_EVENT_INDEX.fetch_add(1, Ordering::Relaxed);
		LCustomEventHandle(index)
	}
}

impl LuaUserData for LCustomEventHandle {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(
			LuaMetaMethod::Eq,
			|_: &Lua, a: &LCustomEventHandle, b: LCustomEventHandle| Ok(a.0 == b.0),
		);
	}
}
