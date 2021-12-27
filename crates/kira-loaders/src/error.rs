use std::fmt::Display;

/// Errors that can occur when loading or streaming an audio file.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	/// Could not determine the default audio track in the file.
	NoDefaultTrack,
	/// Could not determine the sample rate of the audio.
	UnknownSampleRate,
	/// The audio uses an unsupported channel configuration. Only
	/// mono and stereo audio is supported.
	UnsupportedChannelConfiguration,
	/// An error occurred while reading the file from the filesystem.
	IoError(std::io::Error),
	/// An error occurred when parsing the file.
	SymphoniaError(symphonia::core::errors::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoDefaultTrack => f.write_str("Could not determine the default audio track"),
			Error::UnknownSampleRate => {
				f.write_str("Could not detect the sample rate of the audio")
			}
			Error::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
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
