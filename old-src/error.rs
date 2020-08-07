use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ConductorError {
	SendCommand,
}

impl Display for ConductorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			ConductorError::SendCommand => "Error sending a command to the audio thread",
		})
	}
}

impl Error for ConductorError {}
