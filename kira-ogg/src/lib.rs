use std::{
	error::Error,
	fmt::{Display, Formatter},
	io::{Read, Seek},
};

use kira::{
	sound::static_sound::{StaticSound, StaticSoundSettings},
	Frame,
};

#[derive(Debug)]
pub enum FromReaderError {
	UnsupportedChannelConfiguration,
	VorbisError(lewton::VorbisError),
}

impl Display for FromReaderError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FromReaderError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			FromReaderError::VorbisError(error) => error.fmt(f),
		}
	}
}

impl Error for FromReaderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			FromReaderError::VorbisError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<lewton::VorbisError> for FromReaderError {
	fn from(v: lewton::VorbisError) -> Self {
		Self::VorbisError(v)
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

/// Decodes a [`StaticSound`] from an ogg reader.
pub fn from_reader<R>(
	reader: R,
	settings: StaticSoundSettings,
) -> Result<StaticSound, FromReaderError>
where
	R: Read + Seek,
{
	use lewton::{inside_ogg::OggStreamReader, samples::Samples};
	let mut reader = OggStreamReader::new(reader)?;
	let mut stereo_samples = vec![];
	while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
		let num_channels = packet.len();
		let num_samples = packet.num_samples();
		match num_channels {
			1 => {
				for i in 0..num_samples {
					stereo_samples.push(Frame::from_mono(packet[0][i]));
				}
			}
			2 => {
				for i in 0..num_samples {
					stereo_samples.push(Frame::new(packet[0][i], packet[1][i]));
				}
			}
			_ => return Err(FromReaderError::UnsupportedChannelConfiguration),
		}
	}
	Ok(StaticSound::from_frames(
		reader.ident_hdr.audio_sample_rate,
		stereo_samples,
		settings,
	))
}

/// Decodes a [`StaticSound`] from an ogg file.
pub fn from_file<P>(path: P, settings: StaticSoundSettings) -> Result<StaticSound, FromFileError>
where
	P: AsRef<std::path::Path>,
{
	Ok(from_reader(std::fs::File::open(path)?, settings)?)
}
