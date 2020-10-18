use conductor::{metronome::MetronomeSettings, tempo::Tempo};
use mlua::prelude::*;

use crate::{error::ConductorLuaError, tempo::LTempo};

pub struct LMetronomeSettings(pub MetronomeSettings);

impl<'lua> FromLua<'lua> for LMetronomeSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LMetronomeSettings(MetronomeSettings::default())),
			LuaValue::Table(table) => Ok(LMetronomeSettings(MetronomeSettings {
				tempo: table
					.get::<_, Option<LTempo>>("tempo")?
					.map(|tempo| tempo.0)
					.unwrap_or(MetronomeSettings::default().tempo),
				interval_events_to_emit: table
					.get::<_, Option<Vec<f64>>>("intervalEventsToEmit")?
					.unwrap_or(vec![]),
			})),
			_ => Err(ConductorLuaError::wrong_argument_type(
				"metronome settings",
				"table",
			)),
		}
	}
}
