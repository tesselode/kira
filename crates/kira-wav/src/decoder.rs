use std::{collections::VecDeque, fs::File, io::BufReader, path::Path};

use hound::{SampleFormat, WavReader};
use kira::dsp::{Frame, I24};

use crate::{DecodeError, Error};

const MAX_BLOCK_SIZE: usize = 512;

pub(crate) struct Decoder {
	reader: WavReader<BufReader<File>>,
}

impl Decoder {
	pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
		let reader =
			WavReader::new(BufReader::new(File::open(path)?)).map_err(DecodeError::WavError)?;
		if reader.spec().channels > 2 {
			return Err(DecodeError::UnsupportedChannelConfiguration.into());
		}
		if reader.spec().sample_format == SampleFormat::Int
			&& !matches!(reader.spec().bits_per_sample, 16 | 24)
		{
			return Err(DecodeError::UnsupportedBitDepth.into());
		}
		Ok(Self { reader })
	}
}

impl kira_streaming::Decoder for Decoder {
	type Error = Error;

	fn sample_rate(&mut self) -> u32 {
		self.reader.spec().sample_rate
	}

	fn decode(&mut self) -> Result<Option<VecDeque<Frame>>, Self::Error> {
		Ok(match self.reader.spec().channels {
			1 => match self.reader.spec().sample_format {
				SampleFormat::Float => {
					let mut frames = VecDeque::new();
					for _ in 0..MAX_BLOCK_SIZE {
						if let Some(sample) = self.reader.samples::<f32>().next() {
							let sample = sample.map_err(DecodeError::WavError)?;
							frames.push_back(Frame::from(sample));
						} else {
							break;
						}
					}
					if frames.is_empty() {
						None
					} else {
						Some(frames)
					}
				}
				SampleFormat::Int => match self.reader.spec().bits_per_sample {
					16 => {
						let mut frames = VecDeque::new();
						for _ in 0..MAX_BLOCK_SIZE {
							if let Some(sample) = self.reader.samples::<i16>().next() {
								let sample = sample.map_err(DecodeError::WavError)?;
								frames.push_back(Frame::from(sample));
							} else {
								break;
							}
						}
						if frames.is_empty() {
							None
						} else {
							Some(frames)
						}
					}
					24 => {
						let mut frames = VecDeque::new();
						for _ in 0..MAX_BLOCK_SIZE {
							if let Some(sample) = self.reader.samples::<i32>().next() {
								let sample = sample.map_err(DecodeError::WavError)?;
								frames.push_back(Frame::from(I24(sample)));
							} else {
								break;
							}
						}
						if frames.is_empty() {
							None
						} else {
							Some(frames)
						}
					}
					_ => panic!("Unsupported bit depth. This should have been checked when the decoder was created."),
				},
			},
			2 => match self.reader.spec().sample_format {
				SampleFormat::Float => {
					let mut frames = VecDeque::new();
					for _ in 0..MAX_BLOCK_SIZE {
						let mut samples = self.reader.samples::<f32>();
						if let (Some(left), Some(right)) = (samples.next(), samples.next()) {
							let left = left.map_err(DecodeError::WavError)?;
							let right = right.map_err(DecodeError::WavError)?;
							frames.push_back(Frame::from([left, right]));
						} else {
							break;
						}
					}
					if frames.is_empty() {
						None
					} else {
						Some(frames)
					}
				}
				SampleFormat::Int => match self.reader.spec().bits_per_sample {
					16 => {
						let mut frames = VecDeque::new();
						for _ in 0..MAX_BLOCK_SIZE {
							let mut samples = self.reader.samples::<i16>();
							if let (Some(left), Some(right)) = (samples.next(), samples.next()) {
								let left = left.map_err(DecodeError::WavError)?;
								let right = right.map_err(DecodeError::WavError)?;
								frames.push_back(Frame::from([left, right]));
							} else {
								break;
							}
						}
						if frames.is_empty() {
							None
						} else {
							Some(frames)
						}
					}
					24 => {
						let mut frames = VecDeque::new();
						for _ in 0..MAX_BLOCK_SIZE {
							let mut samples = self.reader.samples::<i32>();
							if let (Some(left), Some(right)) = (samples.next(), samples.next()) {
								let left = left.map_err(DecodeError::WavError)?;
								let right = right.map_err(DecodeError::WavError)?;
								frames.push_back(Frame::from([I24(left), I24(right)]));
							} else {
								break;
							}
						}
						if frames.is_empty() {
							None
						} else {
							Some(frames)
						}
					}
					_ => panic!("Unsupported bit depth. This should have been checked when the decoder was created."),
				},
			},
			_ => panic!("Unsupported channel configuration. This should have been checked when the decoder was created."),
		})
	}

	fn reset(&mut self) -> Result<(), Self::Error> {
		self.reader.seek(0)?;
		Ok(())
	}
}
