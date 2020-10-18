use conductor::metronome::MetronomeSettings;
use mlua::prelude::*;

use crate::{error::ConductorLuaError, tempo::LTempo};

pub struct LMetronomeSettings(pub MetronomeSettings);

impl<'lua> FromLua<'lua> for LMetronomeSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LMetronomeSettings(MetronomeSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = MetronomeSettings::default();
				if table.contains_key("tempo")? {
					settings.tempo = table.get::<_, LTempo>("tempo")?.0;
				}
				if table.contains_key("intervalEventsToEmit")? {
					settings.interval_events_to_emit =
						table.get::<_, Vec<f64>>("intervalEventsToEmit")?;
				}
				Ok(LMetronomeSettings(settings))
			}
			value => Err(LuaError::external(ConductorLuaError::wrong_argument_type(
				"metronome settings",
				"table",
				value,
			))),
		}
	}
}
