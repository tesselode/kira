use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ConductorLuaError {
	WrongArgumentType(String, String),
	InvalidDurationUnit,
}

impl ConductorLuaError {
	pub fn wrong_argument_type(thing: &str, correct_type: &str) -> Self {
		Self::WrongArgumentType(thing.into(), correct_type.into())
	}
}

impl Display for ConductorLuaError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ConductorLuaError::WrongArgumentType(thing, correct_type) => {
				f.write_str(&format!("{} must be a {}", thing, correct_type))
			}
			ConductorLuaError::InvalidDurationUnit => {
				f.write_str("duration unit must be 'second(s)' or 'beat(s)'")
			}
		}
	}
}

impl Error for ConductorLuaError {}
