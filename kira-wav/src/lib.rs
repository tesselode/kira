use std::{
	error::Error,
	fmt::{Display, Formatter},
	io::Read,
};

use hound::SampleFormat;
use kira::{
	sound::static_sound::{StaticSound, StaticSoundSettings},
	Frame,
};

#[derive(Debug)]
pub enum FromReaderError {
	UnsupportedChannelConfiguration,
	WavError(hound::Error),
}

impl Display for FromReaderError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FromReaderError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			FromReaderError::WavError(error) => error.fmt(f),
		}
	}
}

impl Error for FromReaderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			FromReaderError::WavError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<hound::Error> for FromReaderError {
	fn from(v: hound::Error) -> Self {
		Self::WavError(v)
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

/// Decodes [`StaticSound`] from a wav reader.
pub fn from_reader<R>(
	reader: R,
	settings: StaticSoundSettings,
) -> Result<StaticSound, FromReaderError>
where
	R: Read,
{
	let mut reader = hound::WavReader::new(reader)?;
	let spec = reader.spec();
	let mut stereo_samples = vec![];
	match reader.spec().channels {
		1 => match spec.sample_format {
			SampleFormat::Float => {
				for sample in reader.samples::<f32>() {
					stereo_samples.push(Frame::from_mono(sample?))
				}
			}
			SampleFormat::Int => {
				for sample in reader.samples::<i32>() {
					let sample = sample?;
					stereo_samples.push(Frame::from_i32(
						sample,
						sample,
						spec.bits_per_sample.into(),
					));
				}
			}
		},
		2 => match spec.sample_format {
			SampleFormat::Float => {
				let mut iter = reader.samples::<f32>();
				while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
					stereo_samples.push(Frame::new(left?, right?));
				}
			}
			SampleFormat::Int => {
				let mut iter = reader.samples::<i32>();
				while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
					stereo_samples.push(Frame::from_i32(
						left?,
						right?,
						spec.bits_per_sample.into(),
					));
				}
			}
		},
		_ => return Err(FromReaderError::UnsupportedChannelConfiguration),
	}
	Ok(StaticSound::from_frames(
		reader.spec().sample_rate,
		stereo_samples,
		settings,
	))
}

/// Decodes a [`StaticSound`] from a wav file.
pub fn from_file<P>(path: P, settings: StaticSoundSettings) -> Result<StaticSound, FromFileError>
where
	P: AsRef<std::path::Path>,
{
	Ok(from_reader(std::fs::File::open(path)?, settings)?)
}
