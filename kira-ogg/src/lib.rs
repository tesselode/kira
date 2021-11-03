use std::{
	fmt::Display,
	fs::File,
	io::{Read, Seek},
	path::Path,
};

use kira::sound::static_sound::{Samples, StaticSound};
use lewton::{inside_ogg::OggStreamReader, VorbisError};

#[derive(Debug)]
pub enum DecodeError {
	UnsupportedChannelConfiguration,
	VorbisError(VorbisError),
}

impl Display for DecodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DecodeError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			DecodeError::VorbisError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for DecodeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			DecodeError::VorbisError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<VorbisError> for DecodeError {
	fn from(v: VorbisError) -> Self {
		Self::VorbisError(v)
	}
}

#[derive(Debug)]
pub enum FromFileError {
	IoError(std::io::Error),
	DecodeError(DecodeError),
}

impl Display for FromFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::DecodeError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for FromFileError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::DecodeError(error) => Some(error),
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<DecodeError> for FromFileError {
	fn from(v: DecodeError) -> Self {
		Self::DecodeError(v)
	}
}

pub fn from_reader(reader: impl Read + Seek) -> Result<StaticSound, DecodeError> {
	let mut reader = OggStreamReader::new(reader)?;
	let samples = match reader.ident_hdr.audio_channels {
		1 => {
			let mut samples = vec![];
			while let Some(mut packet) = reader.read_dec_packet_itl()? {
				samples.append(&mut packet);
			}
			Samples::I16Mono(samples)
		}
		2 => {
			let mut samples = vec![];
			while let Some(packet) = reader.read_dec_packet_itl()? {
				for chunk in packet.chunks_exact(2) {
					samples.push([chunk[0], chunk[1]]);
				}
			}
			Samples::I16Stereo(samples)
		}
		_ => return Err(DecodeError::UnsupportedChannelConfiguration),
	};
	Ok(StaticSound::new(
		reader.ident_hdr.audio_sample_rate,
		samples,
	))
}

fn from_file(path: impl AsRef<Path>) -> Result<StaticSound, FromFileError> {
	Ok(from_reader(File::open(path)?)?)
}
