#![warn(missing_docs)]

mod decoder;
mod error;

pub use error::*;

use kira_streaming::{StreamingSoundData, StreamingSoundSettings};

use std::{
	fs::File,
	io::{Read, Seek},
	path::Path,
	sync::Arc,
};

use kira::sound::static_sound::{Samples, StaticSoundData, StaticSoundSettings};
use lewton::inside_ogg::OggStreamReader;

pub fn load_from_reader(
	reader: impl Read + Seek,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, DecodeError> {
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
		settings,
	})
}

pub fn load_from_file(
	path: impl AsRef<Path>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Error> {
	Ok(load_from_reader(File::open(path)?, settings)?)
}

pub fn stream(
	path: impl AsRef<Path>,
	settings: StreamingSoundSettings,
) -> Result<StreamingSoundData<Error>, Error> {
	Ok(StreamingSoundData::new(
		decoder::Decoder::new(path)?,
		settings,
	))
}
