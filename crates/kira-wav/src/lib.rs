#![warn(missing_docs)]

mod decoder;
mod error;

pub use error::*;
use kira_streaming::{StreamingSoundData, StreamingSoundSettings};

use std::{
	fs::File,
	io::{BufReader, Read},
	path::Path,
	sync::Arc,
};

use hound::{SampleFormat, WavReader};
use kira::{
	dsp::I24,
	sound::static_sound::{Samples, StaticSoundData, StaticSoundSettings},
};

pub fn load_from_reader(
	reader: impl Read,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, DecodeError> {
	let mut reader = WavReader::new(reader)?;
	let samples = match reader.spec().channels {
		1 => match reader.spec().sample_format {
			SampleFormat::Float => Samples::F32Mono(reader.samples().collect::<Result<_, _>>()?),
			SampleFormat::Int => match reader.spec().bits_per_sample {
				16 => Samples::I16Mono(reader.samples().collect::<Result<_, _>>()?),
				24 => Samples::I24Mono(
					reader
						.samples()
						.map(|result| result.map(I24))
						.collect::<Result<_, _>>()?,
				),
				_ => return Err(DecodeError::UnsupportedBitDepth),
			},
		},
		2 => match reader.spec().sample_format {
			SampleFormat::Float => {
				let mut samples = vec![];
				while let (Some(left), Some(right)) =
					(reader.samples().next(), reader.samples().next())
				{
					samples.push([left?, right?]);
				}
				Samples::F32Stereo(samples)
			}
			SampleFormat::Int => match reader.spec().bits_per_sample {
				16 => {
					let mut samples = vec![];
					while let (Some(left), Some(right)) =
						(reader.samples().next(), reader.samples().next())
					{
						samples.push([left?, right?]);
					}
					Samples::I16Stereo(samples)
				}
				24 => {
					let mut samples = vec![];
					while let (Some(left), Some(right)) =
						(reader.samples().next(), reader.samples().next())
					{
						samples.push([I24(left?), I24(right?)]);
					}
					Samples::I24Stereo(samples)
				}
				_ => return Err(DecodeError::UnsupportedBitDepth),
			},
		},
		_ => return Err(DecodeError::UnsupportedChannelConfiguration),
	};
	Ok(StaticSoundData {
		sample_rate: reader.spec().sample_rate,
		samples: Arc::new(samples),
		settings,
	})
}

pub fn load_from_file(
	path: impl AsRef<Path>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Error> {
	Ok(load_from_reader(
		BufReader::new(File::open(path)?),
		settings,
	)?)
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
