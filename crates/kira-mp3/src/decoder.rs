use std::{collections::VecDeque, fs::File, io::Seek, path::Path};

use kira::dsp::Frame;

use crate::{DecodeError, Error};

pub(crate) struct Decoder {
	sample_rate: u32,
	decoder: Option<minimp3::Decoder<File>>,
}

impl Decoder {
	pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
		let path = path.as_ref();
		// decode one frame just to get the sample rate
		let sample_rate = minimp3::Decoder::new(File::open(path)?)
			.next_frame()
			.map(|frame| frame.sample_rate as u32)
			.map_err(DecodeError::Mp3Error)?;
		Ok(Self {
			sample_rate,
			decoder: Some(minimp3::Decoder::new(File::open(path)?)),
		})
	}

	fn decoder_mut(&mut self) -> &mut minimp3::Decoder<File> {
		self.decoder.as_mut().unwrap()
	}
}

impl kira_streaming::Decoder for Decoder {
	type Error = Error;

	fn sample_rate(&mut self) -> u32 {
		self.sample_rate
	}

	fn decode(&mut self) -> Result<Option<VecDeque<Frame>>, Self::Error> {
		match self.decoder_mut().next_frame() {
			Ok(frame) => match frame.channels {
				1 => Ok(Some(
					frame
						.data
						.iter()
						.map(|sample| Frame::from(*sample))
						.collect(),
				)),
				2 => Ok(Some(
					frame
						.data
						.chunks_exact(2)
						.map(|chunk| Frame::from([chunk[0], chunk[1]]))
						.collect(),
				)),
				_ => Err(DecodeError::UnsupportedChannelConfiguration.into()),
			},
			Err(error) => match error {
				minimp3::Error::Eof => Ok(None),
				error => Err(DecodeError::Mp3Error(error).into()),
			},
		}
	}

	fn reset(&mut self) -> Result<(), Self::Error> {
		let mut file = self.decoder.take().unwrap().into_inner();
		file.rewind()?;
		self.decoder = Some(minimp3::Decoder::new(file));
		Ok(())
	}
}
