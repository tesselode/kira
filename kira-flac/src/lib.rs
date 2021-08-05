use std::{
	error::Error,
	fmt::{Display, Formatter},
	io::Read,
};

use kira::{
	sound::static_sound::{StaticSound, StaticSoundSettings},
	Frame,
};

#[derive(Debug)]
pub enum FromReaderError {
	UnsupportedChannelConfiguration,
	FlacError(claxon::Error),
}

impl Display for FromReaderError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FromReaderError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			FromReaderError::FlacError(error) => error.fmt(f),
		}
	}
}

impl Error for FromReaderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			FromReaderError::FlacError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<claxon::Error> for FromReaderError {
	fn from(v: claxon::Error) -> Self {
		Self::FlacError(v)
	}
}

#[derive(Debug)]
pub enum FromFileError {
	IoError(std::io::Error),
	FromReaderError(FromReaderError),
}

impl Display for FromFileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::FromReaderError(error) => error.fmt(f),
		}
	}
}

impl Error for FromFileError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::FromReaderError(error) => Some(error),
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<FromReaderError> for FromFileError {
	fn from(v: FromReaderError) -> Self {
		Self::FromReaderError(v)
	}
}

/// Decodes a [`StaticSound`] from a flac file.
pub fn from_reader<R>(
	reader: R,
	settings: StaticSoundSettings,
) -> Result<StaticSound, FromReaderError>
where
	R: Read,
{
	let mut reader = claxon::FlacReader::new(reader)?;
	let streaminfo = reader.streaminfo();
	let mut stereo_samples = vec![];
	match reader.streaminfo().channels {
		1 => {
			for sample in reader.samples() {
				let sample = sample?;
				stereo_samples.push(Frame::from_i32(sample, sample, streaminfo.bits_per_sample));
			}
		}
		2 => {
			let mut iter = reader.samples();
			while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
				stereo_samples.push(Frame::from_i32(left?, right?, streaminfo.bits_per_sample));
			}
		}
		_ => return Err(FromReaderError::UnsupportedChannelConfiguration),
	}
	Ok(StaticSound::from_frames(
		streaminfo.sample_rate,
		stereo_samples,
		settings,
	))
}

/// Decodes [`StaticSound`] from a flac reader.
pub fn from_file<P>(path: P, settings: StaticSoundSettings) -> Result<StaticSound, FromFileError>
where
	P: AsRef<std::path::Path>,
{
	Ok(from_reader(std::fs::File::open(path)?, settings)?)
}
