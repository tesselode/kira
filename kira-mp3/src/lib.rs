use std::{
	error::Error,
	fmt::{Display, Formatter},
	io::Read,
};

use kira::{
	sound::data::static_sound::{StaticSoundData, StaticSoundDataSettings},
	Frame,
};

#[derive(Debug)]
pub enum FromReaderError {
	UnsupportedChannelConfiguration,
	VariableSampleRate,
	UnknownSampleRate,
	Mp3Error(minimp3::Error),
}

impl Display for FromReaderError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FromReaderError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			FromReaderError::VariableSampleRate => {
				f.write_str("mp3s with variable sample rates are not supported")
			}
			FromReaderError::UnknownSampleRate => {
				f.write_str("Could not get the sample rate of the mp3")
			}
			FromReaderError::Mp3Error(error) => error.fmt(f),
		}
	}
}

impl Error for FromReaderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			FromReaderError::Mp3Error(error) => Some(error),
			_ => None,
		}
	}
}

impl From<minimp3::Error> for FromReaderError {
	fn from(v: minimp3::Error) -> Self {
		Self::Mp3Error(v)
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

/// Decodes a [`StaticSoundData`] from an mp3 reader.
pub fn from_reader<R>(
	reader: R,
	settings: StaticSoundDataSettings,
) -> Result<StaticSoundData, FromReaderError>
where
	R: Read,
{
	let mut decoder = minimp3::Decoder::new(reader);
	let mut sample_rate = None;
	let mut stereo_samples = vec![];
	loop {
		match decoder.next_frame() {
			Ok(frame) => {
				if let Some(sample_rate) = sample_rate {
					if sample_rate != frame.sample_rate {
						return Err(FromReaderError::VariableSampleRate);
					}
				} else {
					sample_rate = Some(frame.sample_rate);
				}
				match frame.channels {
					1 => {
						for sample in frame.data {
							stereo_samples.push(Frame::from_i32(sample.into(), sample.into(), 16))
						}
					}
					2 => {
						let mut iter = frame.data.iter();
						while let (Some(left), Some(right)) = (iter.next(), iter.next()) {
							stereo_samples.push(Frame::from_i32(
								(*left).into(),
								(*right).into(),
								16,
							))
						}
					}
					_ => return Err(FromReaderError::UnsupportedChannelConfiguration),
				}
			}
			Err(error) => match error {
				minimp3::Error::Eof => break,
				error => return Err(error.into()),
			},
		}
	}
	let sample_rate = match sample_rate {
		Some(sample_rate) => sample_rate,
		None => return Err(FromReaderError::UnknownSampleRate),
	};
	Ok(StaticSoundData::from_frames(
		sample_rate as u32,
		stereo_samples,
		settings,
	))
}

/// Decodes a [`StaticSoundData`] from an mp3 file.
pub fn from_file<P>(
	path: P,
	settings: StaticSoundDataSettings,
) -> Result<StaticSoundData, FromFileError>
where
	P: AsRef<std::path::Path>,
{
	Ok(from_reader(std::fs::File::open(path)?, settings)?)
}
