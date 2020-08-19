use conductor::instance::{InstanceId, InstanceSettings};
use mlua::prelude::*;

#[derive(Clone)]
pub struct LInstanceId(pub InstanceId);

impl LuaUserData for LInstanceId {}

pub struct LInstanceSettings(pub InstanceSettings);

impl<'lua> FromLua<'lua> for LInstanceSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		Ok(LInstanceSettings(match lua_value {
			LuaNil => InstanceSettings::default(),
			LuaValue::Table(table) => {
				let mut settings = InstanceSettings::default();
				if table.contains_key("volume")? {
					settings.volume = table.get("volume")?;
				}
				if table.contains_key("pitch")? {
					settings.pitch = table.get("pitch")?;
				}
				if table.contains_key("position")? {
					settings.position = table.get("position")?;
				}
				if table.contains_key("fadeInDuration")? {
					settings.fade_in_duration = Some(table.get("fadeInDuration")?);
				}
				settings
			}
			_ => panic!(),
		}))
	}
}
