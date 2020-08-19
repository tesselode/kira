use conductor::{metronome::MetronomeSettings, tempo::Tempo};
use mlua::prelude::*;

pub struct LMetronomeSettings(pub MetronomeSettings);

impl LuaUserData for LMetronomeSettings {}

impl<'lua> FromLua<'lua> for LMetronomeSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LMetronomeSettings(MetronomeSettings::default())),
			LuaValue::Table(table) => {
				let mut metronome_settings = MetronomeSettings::default();
				if table.contains_key("tempo")? {
					metronome_settings.tempo = Tempo(table.get("tempo")?);
				}
				if table.contains_key("intervalEventsToEmit")? {
					metronome_settings.interval_events_to_emit =
						table.get("intervalEventsToEmit")?;
				}
				Ok(LMetronomeSettings(metronome_settings))
			}
			_ => panic!(),
		}
	}
}
