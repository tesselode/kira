use std::fmt::Display;

/// Errors that can occur when decoding mp3 audio.
#[derive(Debug)]
pub enum DecodeError {
	/// The audio has an unsupported channel configuration. Only
	/// mono and stereo audio is supported.
	UnsupportedChannelConfiguration,
	/// The audio has a variable sample rate, which is not
	/// supported.
	VariableSampleRate,
	/// An error occurred in the decoding process.
	Mp3Error(minimp3::Error),
}

impl Display for DecodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DecodeError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			DecodeError::VariableSampleRate => {
				f.write_str("The audio has a variable sample rate, which is not supported")
			}
			DecodeError::Mp3Error(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for DecodeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			DecodeError::Mp3Error(error) => Some(error),
			_ => None,
		}
	}
}

impl From<minimp3::Error> for DecodeError {
	fn from(v: minimp3::Error) -> Self {
		Self::Mp3Error(v)
	}
}

/// Errors that can occur when loading or streaming mp3 files.
#[derive(Debug)]
pub enum Error {
	/// An error occurred when reading the file.
	IoError(std::io::Error),
	/// An error occurred when decoding the audio.
	DecodeError(DecodeError),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::IoError(error) => error.fmt(f),
			Error::DecodeError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::IoError(error) => Some(error),
			Error::DecodeError(error) => Some(error),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<DecodeError> for Error {
	fn from(v: DecodeError) -> Self {
		Self::DecodeError(v)
	}
}
