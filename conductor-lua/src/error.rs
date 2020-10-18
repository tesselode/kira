use mlua::prelude::*;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ConductorLuaError {
	WrongArgumentType(String, String),
}

impl ConductorLuaError {
	pub fn wrong_argument_type(thing: &str, correct_type: &str) -> LuaError {
		Self::WrongArgumentType(thing.into(), correct_type.into()).into()
	}
}

impl Display for ConductorLuaError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ConductorLuaError::WrongArgumentType(thing, correct_type) => {
				f.write_str(&format!("{} must be a {}", thing, correct_type))
			}
		}
	}
}

impl Error for ConductorLuaError {}

impl Into<LuaError> for ConductorLuaError {
	fn into(self) -> LuaError {
		LuaError::external(self)
	}
}
