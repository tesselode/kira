use std::{collections::VecDeque, fs::File, io::Seek, path::Path};

use claxon::FlacReader;
use kira::dsp::{Frame, I24};

use crate::{DecodeError, Error};

pub(crate) struct Decoder {
	reader: Option<FlacReader<File>>,
}

impl Decoder {
	pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
		let reader = FlacReader::new(File::open(path)?).map_err(DecodeError::FlacError)?;
		if reader.streaminfo().channels > 2 {
			return Err(DecodeError::UnsupportedChannelConfiguration.into());
		}
		if !matches!(reader.streaminfo().bits_per_sample, 16 | 24) {
			return Err(DecodeError::UnsupportedBitDepth.into());
		}
		Ok(Self {
			reader: Some(reader),
		})
	}

	fn reader_mut(&mut self) -> &mut FlacReader<File> {
		self.reader.as_mut().unwrap()
	}
}

impl kira_streaming::Decoder for Decoder {
	type Error = Error;

	fn sample_rate(&mut self) -> u32 {
		self.reader_mut().streaminfo().sample_rate
	}

	fn decode(&mut self) -> Result<Option<VecDeque<Frame>>, Self::Error> {
		let bit_depth = self.reader_mut().streaminfo().bits_per_sample;
		Ok(self
			.reader_mut()
			.blocks()
			.read_next_or_eof(vec![])
			.map_err(DecodeError::FlacError)?
			.map(|block| match block.channels() {
				1 => match bit_depth {
					16 => block
						.channel(0)
						.iter()
						.map(|sample| Frame::from(*sample as i16))
						.collect(),
					24 => block
						.channel(0)
						.iter()
						.map(|sample| Frame::from(I24(*sample)))
						.collect(),
					_ => panic!("Unsupported bit depth. This should have been checked when the decoder was created."),
				},
				2 => match bit_depth {
					16 => block
						.stereo_samples()
						.map(|sample| Frame::from([sample.0 as i16, sample.1 as i16]))
						.collect(),
					24 => block
						.stereo_samples()
						.map(|sample| Frame::from([I24(sample.0), I24(sample.1)]))
						.collect(),
					_ => panic!("Unsupported bit depth. This should have been checked when the decoder was created."),
				},
				_ => panic!("Unsupported channel configuration. This should have been checked when the decoder was created."),
			}))
	}

	fn reset(&mut self) -> Result<(), Self::Error> {
		let mut file = self.reader.take().unwrap().into_inner();
		file.rewind()?;
		self.reader = Some(FlacReader::new(file).map_err(DecodeError::FlacError)?);
		Ok(())
	}
}
