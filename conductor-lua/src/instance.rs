use conductor::instance::InstanceId;
use mlua::prelude::*;

pub struct LInstanceId(pub InstanceId);

impl LuaUserData for LInstanceId {}
