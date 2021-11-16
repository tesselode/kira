#![warn(missing_docs)]

mod decoder;
mod error;

pub use error::*;
use kira_streaming::{StreamingSoundData, StreamingSoundSettings};

use std::{fs::File, io::Read, path::Path, sync::Arc};

use claxon::FlacReader;
use kira::{
	dsp::I24,
	sound::static_sound::{Samples, StaticSoundData, StaticSoundSettings},
};

pub fn load_from_reader(
	reader: impl Read + 'static,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, DecodeError> {
	let mut reader = FlacReader::new(reader)?;
	let samples = match reader.streaminfo().channels {
		1 => match reader.streaminfo().bits_per_sample {
			16 => {
				let mut samples = vec![];
				let mut block_reader = reader.blocks();
				let mut buffer = vec![];
				while let Some(block) = block_reader.read_next_or_eof(buffer)? {
					samples.extend(block.channel(0).iter().map(|sample| *sample as i16));
					buffer = block.into_buffer();
				}
				Samples::I16Mono(samples)
			}
			24 => {
				let mut samples = vec![];
				let mut block_reader = reader.blocks();
				let mut buffer = vec![];
				while let Some(block) = block_reader.read_next_or_eof(buffer)? {
					samples.extend(block.channel(0).iter().copied().map(I24));
					buffer = block.into_buffer();
				}
				Samples::I24Mono(samples)
			}
			_ => return Err(DecodeError::UnsupportedBitDepth),
		},
		2 => match reader.streaminfo().bits_per_sample {
			16 => {
				let mut samples = vec![];
				let mut block_reader = reader.blocks();
				let mut buffer = vec![];
				while let Some(block) = block_reader.read_next_or_eof(buffer)? {
					samples.extend(
						block
							.stereo_samples()
							.map(|sample| [sample.0 as i16, sample.1 as i16]),
					);
					buffer = block.into_buffer();
				}
				Samples::I16Stereo(samples)
			}
			24 => {
				let mut samples = vec![];
				let mut block_reader = reader.blocks();
				let mut buffer = vec![];
				while let Some(block) = block_reader.read_next_or_eof(buffer)? {
					samples.extend(
						block
							.stereo_samples()
							.map(|sample| [I24(sample.0), I24(sample.1)]),
					);
					buffer = block.into_buffer();
				}
				Samples::I24Stereo(samples)
			}
			_ => return Err(DecodeError::UnsupportedBitDepth),
		},
		_ => return Err(DecodeError::UnsupportedChannelConfiguration),
	};
	Ok(StaticSoundData {
		sample_rate: reader.streaminfo().sample_rate,
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
