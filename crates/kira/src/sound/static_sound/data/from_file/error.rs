use std::fmt::Display;

/// Errors that can occur when loading or streaming an audio file.
#[derive(Debug)]
#[non_exhaustive]
pub enum FromFileError {
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

impl Display for FromFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::NoDefaultTrack => {
				f.write_str("Could not determine the default audio track")
			}
			FromFileError::UnknownSampleRate => {
				f.write_str("Could not detect the sample rate of the audio")
			}
			FromFileError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::SymphoniaError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for FromFileError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::SymphoniaError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<symphonia::core::errors::Error> for FromFileError {
	fn from(v: symphonia::core::errors::Error) -> Self {
		Self::SymphoniaError(v)
	}
}
