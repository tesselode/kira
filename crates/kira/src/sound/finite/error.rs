use std::fmt::Display;

/// Errors that can occur when loading or streaming an audio file.
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(
	docsrs,
	doc(cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav")))
)]
pub enum LoadError {
	/// Could not determine the default audio track in the file.
	NoDefaultTrack,
	/// Could not determine the sample rate of the audio.
	UnknownSampleRate,
	/// The audio uses an unsupported channel configuration. Only
	/// mono and stereo audio is supported.
	UnsupportedChannelConfiguration,
	/// An error occurred when parsing the file.
	SymphoniaError(symphonia::core::errors::Error),
}

impl Display for LoadError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LoadError::NoDefaultTrack => f.write_str("Could not determine the default audio track"),
			LoadError::UnknownSampleRate => {
				f.write_str("Could not detect the sample rate of the audio")
			}
			LoadError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			LoadError::SymphoniaError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for LoadError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			LoadError::SymphoniaError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<symphonia::core::errors::Error> for LoadError {
	fn from(v: symphonia::core::errors::Error) -> Self {
		Self::SymphoniaError(v)
	}
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum FromFileError {
	IoError(std::io::Error),
	LoadError(LoadError),
}

impl Display for FromFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::LoadError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for FromFileError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::LoadError(error) => Some(error),
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<LoadError> for FromFileError {
	fn from(v: LoadError) -> Self {
		Self::LoadError(v)
	}
}
