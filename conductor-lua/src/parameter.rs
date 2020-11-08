use conductor::ParameterId;
use mlua::prelude::*;

#[derive(Debug, Clone)]
pub struct LParameterId(pub ParameterId);

impl LuaUserData for LParameterId {}
