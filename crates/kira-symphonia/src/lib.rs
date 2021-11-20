mod streaming;

pub use streaming::*;

use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
	NoDefaultTrack,
	UnknownSampleRate,
	IoError(std::io::Error),
	SymphoniaError(symphonia::core::errors::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoDefaultTrack => f.write_str("Could not determine the default audio track"),
			Error::UnknownSampleRate => {
				f.write_str("Could not detect the sample rate of the audio")
			}
			Error::IoError(error) => error.fmt(f),
			Error::SymphoniaError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::IoError(error) => Some(error),
			Error::SymphoniaError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<symphonia::core::errors::Error> for Error {
	fn from(v: symphonia::core::errors::Error) -> Self {
		Self::SymphoniaError(v)
	}
}
