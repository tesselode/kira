use mlua::prelude::*;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum LConductorError {
	WrongArgumentType(String, String),
}

impl LConductorError {
	pub fn wrong_argument_type(thing: &str, correct_type: &str) -> LuaError {
		Self::WrongArgumentType(thing.into(), correct_type.into()).into()
	}
}

impl Display for LConductorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LConductorError::WrongArgumentType(thing, correct_type) => {
				f.write_str(&format!("{} must be a {}", thing, correct_type))
			}
		}
	}
}

impl Error for LConductorError {}

impl Into<LuaError> for LConductorError {
	fn into(self) -> LuaError {
		LuaError::external(self)
	}
}
