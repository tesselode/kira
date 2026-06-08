use std::fmt::{Display, Formatter};
/// Errors that can occur when using the cpal backend.
#[derive(Debug)]
pub enum Error {
	/// A default audio output device could not be determined.
	NoDefaultOutputDevice,
	/// An error occurred in the cpal backend.
	CpalError(cpal::Error),
}
impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoDefaultOutputDevice => {
				f.write_str("Cannot find the default audio output device")
			}
			Error::CpalError(error) => error.fmt(f),
		}
	}
}
impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::CpalError(error) => Some(error),
			_ => None,
		}
	}
}
impl From<cpal::Error> for Error {
	fn from(v: cpal::Error) -> Self {
		Self::CpalError(v)
	}
}
