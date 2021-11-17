use std::fmt::Display;

/// Errors that can occur when decoding wav audio.
#[derive(Debug)]
pub enum DecodeError {
	/// The audio is an unsupported bit depth. Only 16 and 24-bit
	/// audio are supported.
	UnsupportedBitDepth,
	/// The audio has an unsupported channel configuration. Only
	/// mono and stereo audio is supported.
	UnsupportedChannelConfiguration,
	/// An error occurred in the decoding process.
	WavError(hound::Error),
}

impl Display for DecodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DecodeError::UnsupportedBitDepth => {
				f.write_str("Only 16 and 24-bit audio is supported")
			}
			DecodeError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			DecodeError::WavError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for DecodeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			DecodeError::WavError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<hound::Error> for DecodeError {
	fn from(v: hound::Error) -> Self {
		Self::WavError(v)
	}
}

/// Errors that can occur when loading or streaming wav files.
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
