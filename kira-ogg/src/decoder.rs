use std::{
	error::Error,
	fmt::{Display, Formatter},
	fs::File,
	io::Seek,
	ops::Range,
	path::Path,
};

use kira::Frame;
use lewton::{inside_ogg::OggStreamReader, VorbisError};

#[derive(Debug)]
pub enum DecoderError {
	UnsupportedChannelConfiguration,
	IoError(std::io::Error),
	VorbisError(VorbisError),
}

impl Display for DecoderError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DecoderError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			DecoderError::IoError(error) => error.fmt(f),
			DecoderError::VorbisError(error) => error.fmt(f),
		}
	}
}

impl Error for DecoderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			DecoderError::IoError(error) => Some(error),
			DecoderError::VorbisError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<std::io::Error> for DecoderError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<VorbisError> for DecoderError {
	fn from(v: VorbisError) -> Self {
		Self::VorbisError(v)
	}
}

struct CurrentPacket {
	packet: Vec<Frame>,
	relative_frame_index: usize,
}

pub struct Decoder {
	frame_count: usize,
	reader: Option<OggStreamReader<File>>,
	frame_index: usize,
	current_packet: Option<CurrentPacket>,
}

impl Decoder {
	pub fn new(path: impl AsRef<Path>) -> Result<Self, DecoderError> {
		let mut reader = OggStreamReader::new(File::open(path)?)?;
		let channels = reader.ident_hdr.audio_channels;
		if !matches!(channels, 1 | 2) {
			return Err(DecoderError::UnsupportedChannelConfiguration);
		}
		let mut frame_count = 0;
		while let Some(packet) = reader.read_dec_packet_itl()? {
			frame_count += packet.chunks_exact(channels.into()).count();
		}
		let mut file = reader.into_inner().into_inner();
		file.rewind()?;
		let reader = OggStreamReader::new(file)?;
		Ok(Self {
			frame_count,
			reader: Some(reader),
			frame_index: 0,
			current_packet: None,
		})
	}

	fn reader(&mut self) -> &mut OggStreamReader<File> {
		self.reader.as_mut().unwrap()
	}

	fn ensure_packet(&mut self) -> Result<(), VorbisError> {
		if self.current_packet.is_some() {
			return Ok(());
		}
		while let Some(samples) = self.reader().read_dec_packet_itl()? {
			if samples.is_empty() {
				continue;
			}
			self.current_packet = Some(CurrentPacket {
				packet: match self.reader().ident_hdr.audio_channels {
					1 => samples
						.iter()
						.map(|sample| Frame::from_i32((*sample).into(), (*sample).into(), 16))
						.collect(),
					2 => samples
						.chunks_exact(2)
						.map(|chunk| Frame::from_i32(chunk[0].into(), chunk[1].into(), 16))
						.collect(),
					_ => unreachable!(),
				},
				relative_frame_index: 0,
			});
			break;
		}
		Ok(())
	}

	fn reset(&mut self) -> Result<(), VorbisError> {
		let reader = self.reader.take().unwrap();
		let mut file = reader.into_inner().into_inner();
		file.rewind().unwrap();
		self.reader = Some(OggStreamReader::new(file)?);
		self.frame_index = 0;
		Ok(())
	}
}

impl kira::sound::streaming::Decoder for Decoder {
	fn sample_rate(&mut self) -> u32 {
		self.reader().ident_hdr.audio_sample_rate
	}

	fn frame_count(&mut self) -> usize {
		self.frame_count
	}

	fn decode(&mut self, frame_indices: Range<usize>) -> Vec<Frame> {
		let mut frames = Vec::with_capacity(frame_indices.end - frame_indices.start);
		if self.frame_index > frame_indices.start {
			self.reset().unwrap();
		}
		loop {
			self.ensure_packet().unwrap();
			let CurrentPacket { packet, relative_frame_index } = self.current_packet.as_mut().expect(
				"ensure_packet should have either set self.current_packet to Some or returned an error"
			);
			if frame_indices.contains(&self.frame_index) {
				frames.push(packet[*relative_frame_index]);
			}
			*relative_frame_index += 1;
			if *relative_frame_index >= packet.len() {
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
