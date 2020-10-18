use std::sync::atomic::{AtomicUsize, Ordering};

use conductor::manager::Event;
use mlua::prelude::*;

static NEXT_EVENT_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone)]
pub struct CustomEvent(pub usize);

impl CustomEvent {
	pub fn new() -> Self {
		let index = NEXT_EVENT_INDEX.fetch_add(1, Ordering::Relaxed);
		CustomEvent(index)
	}
}

impl LuaUserData for CustomEvent {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::Eq, |_: &Lua, this: &Self, other: Self| {
			Ok(this.0 == other.0)
		})
	}
}

pub struct LEvent(pub Event<CustomEvent>);

impl<'lua> ToLua<'lua> for LEvent {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		let table = lua.create_table()?;
		match self.0 {
			Event::MetronomeIntervalPassed(interval) => {
				table.set("kind", "metronomeIntervalPassed")?;
				table.set("interval", interval)?;
			}
			Event::Custom(event) => {
				table.set("kind", "custom")?;
				table.set("event", event)?;
			}
		}
		Ok(LuaValue::Table(table))
	}
}
