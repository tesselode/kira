use std::sync::atomic::{AtomicUsize, Ordering};

use mlua::prelude::*;

static NEXT_EVENT_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone)]
pub struct LEvent(pub usize);

impl LEvent {
	pub fn new() -> Self {
		let index = NEXT_EVENT_INDEX.fetch_add(1, Ordering::Relaxed);
		LEvent(index)
	}
}

impl LuaUserData for LEvent {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::Eq, |_: &Lua, this: &Self, other: Self| {
			Ok(this.0 == other.0)
		})
	}
}
