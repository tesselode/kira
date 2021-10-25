use std::{
	error::Error,
	fmt::{Display, Formatter},
	fs::File,
	io::Seek,
	ops::Range,
	path::Path,
};

use kira::Frame;

#[derive(Debug)]
pub enum DecoderError {
	UnsupportedChannelConfiguration,
	VariableSampleRate,
	UnknownSampleRate,
	IoError(std::io::Error),
	Mp3Error(minimp3::Error),
}

impl Display for DecoderError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DecoderError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			DecoderError::VariableSampleRate => {
				f.write_str("mp3s with variable sample rates are not supported")
			}
			DecoderError::UnknownSampleRate => {
				f.write_str("Could not get the sample rate of the mp3")
			}
			DecoderError::IoError(error) => error.fmt(f),
			DecoderError::Mp3Error(error) => error.fmt(f),
		}
	}
}

impl Error for DecoderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			DecoderError::IoError(error) => Some(error),
			DecoderError::Mp3Error(error) => Some(error),
			_ => None,
		}
	}
}

impl From<std::io::Error> for DecoderError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<minimp3::Error> for DecoderError {
	fn from(v: minimp3::Error) -> Self {
		Self::Mp3Error(v)
	}
}

struct CurrentPacket {
	packet: minimp3::Frame,
	relative_frame_index: usize,
}

pub struct Decoder {
	sample_rate: u32,
	frame_count: usize,
	decoder: Option<minimp3::Decoder<File>>,
	frame_index: usize,
	current_packet: Option<CurrentPacket>,
}

impl Decoder {
	pub fn new(path: impl AsRef<Path>) -> Result<Self, DecoderError> {
		let mut decoder = minimp3::Decoder::new(File::open(path)?);
		let mut sample_rate = None;
		let mut frame_count = 0;
		loop {
			let packet = match decoder.next_frame() {
				Ok(packet) => packet,
				Err(err) => match err {
					minimp3::Error::Io(err) => return Err(DecoderError::IoError(err)),
					minimp3::Error::SkippedData => continue,
					minimp3::Error::Eof => break,
					err => return Err(err.into()),
				},
			};
			if let Some(previous_sample_rate) = sample_rate {
				if packet.sample_rate as u32 != previous_sample_rate {
					return Err(DecoderError::VariableSampleRate);
				}
			} else {
				sample_rate = Some(packet.sample_rate as u32);
			}
			if packet.channels > 2 {
				return Err(DecoderError::UnsupportedChannelConfiguration);
			}
			frame_count += packet.data.len() / packet.channels;
		}
		let sample_rate = sample_rate.ok_or(DecoderError::UnknownSampleRate)?;
		let mut file = decoder.into_inner();
		file.rewind()?;
		let decoder = minimp3::Decoder::new(file);
		Ok(Self {
			sample_rate,
			frame_count,
			decoder: Some(decoder),
			frame_index: 0,
			current_packet: None,
		})
	}

	fn decoder(&mut self) -> &mut minimp3::Decoder<File> {
		self.decoder.as_mut().unwrap()
	}

	fn ensure_packet(&mut self) -> Result<(), minimp3::Error> {
		if self.current_packet.is_some() {
			return Ok(());
		}
		loop {
			match self.decoder().next_frame() {
				Ok(packet) => {
					self.current_packet = Some(CurrentPacket {
						packet,
						relative_frame_index: 0,
					});
					return Ok(());
				}
				Err(err) => match err {
					minimp3::Error::SkippedData => continue,
					err => return Err(err),
				},
			}
		}
	}

	fn reset(&mut self) {
		let decoder = self.decoder.take().unwrap();
		let mut file = decoder.into_inner();
		file.rewind().unwrap();
		self.decoder = Some(minimp3::Decoder::new(file));
		self.frame_index = 0;
	}
}

impl kira::sound::streaming::Decoder for Decoder {
	fn sample_rate(&mut self) -> u32 {
		self.sample_rate
	}

	fn frame_count(&mut self) -> usize {
		self.frame_count
	}

	fn decode(&mut self, frame_indices: Range<usize>) -> Vec<Frame> {
		let mut frames = Vec::with_capacity(frame_indices.end - frame_indices.start);
		if self.frame_index > frame_indices.start {
			self.reset();
		}
		loop {
			self.ensure_packet().unwrap();
			let CurrentPacket { packet, relative_frame_index } = self.current_packet.as_mut().expect(
				"ensure_packet should have either set self.current_packet to Some or returned an error"
			);
			if frame_indices.contains(&self.frame_index) {
				frames.push(frame_from_packet(packet, *relative_frame_index));
			}
			*relative_frame_index += 1;
			if *relative_frame_index >= packet.data.len() / packet.channels {
				self.current_packet = None;
			}
			self.frame_index += 1;
			if self.frame_index >= frame_indices.end {
				break;
			}
		}
		frames
	}
}

fn frame_from_packet(packet: &minimp3::Frame, relative_index: usize) -> Frame {
	match packet.channels {
		1 => {
			let sample = packet.data[relative_index];
			Frame::from_i32(sample.into(), sample.into(), 16)
		}
		2 => {
			let left = packet.data[relative_index * 2];
			let right = packet.data[relative_index * 2 + 1];
			Frame::from_i32(left.into(), right.into(), 16)
		}
		_ => {
			panic!("Unsupported channel configuration - Decoder::new should have returned an error")
		}
	}
}
