use conductor::track::{id::SubTrackId, index::TrackIndex};
use mlua::prelude::*;

use crate::error::ConductorLuaError;

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
		Err(LuaError::external(ConductorLuaError::wrong_argument_type(
			"value",
			"'main' or trackId",
			lua_value,
		)))
	}
}
