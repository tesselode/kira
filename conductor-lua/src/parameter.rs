use conductor::parameter::ParameterId;
use mlua::prelude::*;

pub struct LParameterId(pub ParameterId);

impl LuaUserData for LParameterId {}
