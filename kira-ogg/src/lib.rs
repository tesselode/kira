use std::{
	error::Error,
	fmt::{Display, Formatter},
};

use kira::{
	sound::data::static_sound::{StaticSoundData, StaticSoundDataSettings},
	Frame,
};

#[derive(Debug)]
pub enum OggError {
	UnsupportedChannelConfiguration,
	VorbisError(lewton::VorbisError),
}

impl Display for OggError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			OggError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			OggError::VorbisError(error) => error.fmt(f),
		}
	}
}

impl Error for OggError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			OggError::VorbisError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<lewton::VorbisError> for OggError {
	fn from(v: lewton::VorbisError) -> Self {
		Self::VorbisError(v)
	}
}

#[derive(Debug)]
pub enum FromFileError {
	IoError(std::io::Error),
	OggError(OggError),
}

impl Display for FromFileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::OggError(error) => error.fmt(f),
		}
	}
}

impl Error for FromFileError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::OggError(error) => Some(error),
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<OggError> for FromFileError {
	fn from(v: OggError) -> Self {
		Self::OggError(v)
	}
}

/// Decodes a [`StaticSoundData`] from an ogg reader.
pub fn from_ogg_reader<R>(
	reader: R,
	settings: StaticSoundDataSettings,
) -> Result<StaticSoundData, OggError>
where
	R: std::io::Read + std::io::Seek,
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
			_ => return Err(OggError::UnsupportedChannelConfiguration),
		}
	}
	Ok(StaticSoundData::from_frames(
		reader.ident_hdr.audio_sample_rate,
		stereo_samples,
		settings,
	))
}

/// Decodes a [`StaticSoundData`] from an ogg file.
pub fn from_ogg_file<P>(
	path: P,
	settings: StaticSoundDataSettings,
) -> Result<StaticSoundData, FromFileError>
where
	P: AsRef<std::path::Path>,
{
	Ok(from_ogg_reader(std::fs::File::open(path)?, settings)?)
}
