use kira::mixer::{SubTrackId, TrackIndex, TrackSettings};
use mlua::prelude::*;

use crate::error::KiraLuaError;

pub struct LTrackSettings(pub TrackSettings);

impl<'lua> FromLua<'lua> for LTrackSettings {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match lua_value {
			LuaNil => Ok(LTrackSettings(TrackSettings::default())),
			LuaValue::Table(table) => {
				let mut settings = TrackSettings::default();
				if table.contains_key("volume")? {
					settings.volume = table.get("volume")?;
				}
				Ok(LTrackSettings(settings))
			}
			value => Err(LuaError::external(KiraLuaError::wrong_argument_type(
				"trackSettings",
				"table",
				value,
			))),
		}
	}
}

#[derive(Debug, Clone)]
pub struct LSubTrackId(pub SubTrackId);

impl LuaUserData for LSubTrackId {}

pub struct LTrackIndex(pub TrackIndex);

impl<'lua> FromLua<'lua> for LTrackIndex {
	fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
		match &lua_value {
			LuaValue::String(string) => {
				if string.to_str()? == "main" {
					return Ok(LTrackIndex(TrackIndex::Main));
				}
			}
			LuaValue::UserData(user_data) => {
				if user_data.is::<LSubTrackId>() {
					let sub_track_id = user_data.borrow::<LSubTrackId>()?;
					return Ok(LTrackIndex(TrackIndex::Sub(sub_track_id.0)));
				}
			}
			_ => {}
		}
		Err(LuaError::external(KiraLuaError::wrong_argument_type(
			"value",
			"'main' or trackId",
			lua_value,
		)))
	}
}
