use std::{collections::VecDeque, fs::File, io::Seek, path::Path};

use kira::dsp::{Frame, Sample};
use lewton::inside_ogg::OggStreamReader;

use crate::{DecodeError, Error};

pub(crate) struct Decoder {
	reader: Option<OggStreamReader<File>>,
}

impl Decoder {
	pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
		let reader = OggStreamReader::new(File::open(path)?).map_err(DecodeError::VorbisError)?;
		if reader.ident_hdr.audio_channels > 2 {
			return Err(DecodeError::UnsupportedChannelConfiguration.into());
		}
		Ok(Self {
			reader: Some(reader),
		})
	}

	fn reader_mut(&mut self) -> &mut OggStreamReader<File> {
		self.reader.as_mut().unwrap()
	}
}

impl kira_streaming::Decoder for Decoder {
	type Error = Error;

	fn sample_rate(&mut self) -> u32 {
		self.reader_mut().ident_hdr.audio_sample_rate
	}

	fn decode(&mut self) -> Result<Option<VecDeque<Frame>>, Self::Error> {
		Ok(self.reader_mut().read_dec_packet_itl().map_err(DecodeError::VorbisError)?.map(|packet| {
			match self.reader_mut().ident_hdr.audio_channels {
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
		}))
	}

	fn reset(&mut self) -> Result<(), Self::Error> {
		let mut file = self.reader.take().unwrap().into_inner().into_inner();
		file.rewind()?;
		self.reader = Some(OggStreamReader::new(file).map_err(DecodeError::VorbisError)?);
		Ok(())
	}
}
