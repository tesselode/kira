use std::{
	collections::VecDeque,
	fmt::Display,
	fs::File,
	io::{Read, Seek},
	path::Path,
	sync::Arc,
};

use kira::{
	dsp::{Frame, Sample},
	sound::static_sound::{Samples, StaticSoundData},
};
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

pub fn from_reader(reader: impl Read + Seek) -> Result<StaticSoundData, DecodeError> {
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
	Ok(StaticSoundData {
		sample_rate: reader.ident_hdr.audio_sample_rate,
		samples: Arc::new(samples),
	})
}

pub fn from_file(path: impl AsRef<Path>) -> Result<StaticSoundData, FromFileError> {
	Ok(from_reader(File::open(path)?)?)
}

pub struct Decoder {
	reader: OggStreamReader<File>,
}

impl Decoder {
	pub fn new(path: impl AsRef<Path>) -> Result<Self, FromFileError> {
		let reader = OggStreamReader::new(File::open(path)?).map_err(DecodeError::VorbisError)?;
		if reader.ident_hdr.audio_channels > 2 {
			return Err(DecodeError::UnsupportedChannelConfiguration.into());
		}
		Ok(Self { reader })
	}
}

impl kira::sound::streaming::Decoder for Decoder {
	fn sample_rate(&mut self) -> u32 {
		self.reader.ident_hdr.audio_sample_rate
	}

	fn decode(&mut self) -> Option<VecDeque<Frame>> {
		self.reader.read_dec_packet_itl().unwrap().map(|packet| {
			match self.reader.ident_hdr.audio_channels {
				1 => {
					packet.iter().map(|sample| Frame::from_mono(sample.into_f32())).collect()
				},
				2 => {
					packet
						.chunks_exact(2)
						.map(|chunk| Frame::new(chunk[0].into_f32(), chunk[1].into_f32()))
						.collect()
				},
				_ => panic!("Unsupported channel configuration. This should have been checked when the decoder was created.")
			}
		})
	}
}
