use std::{fs::File, path::Path};

use kira::sound::SoundData;
use ringbuf::RingBuffer;
use symphonia::core::{codecs::Decoder, formats::FormatReader, io::MediaSourceStream, probe::Hint};

use crate::Error;

use super::sound::StreamingSound;

const ERROR_BUFFER_CAPACITY: usize = 8;

pub struct StreamingSoundData {
	pub(crate) format_reader: Box<dyn FormatReader>,
	pub(crate) decoder: Box<dyn Decoder>,
}

impl StreamingSoundData {
	pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
		let codecs = symphonia::default::get_codecs();
		let probe = symphonia::default::get_probe();
		let file = File::open(path)?;
		let mss = MediaSourceStream::new(Box::new(file), Default::default());
		let format_reader = probe
			.format(
				&Hint::default(),
				mss,
				&Default::default(),
				&Default::default(),
			)?
			.format;
		let default_track = format_reader.default_track().ok_or(Error::NoDefaultTrack)?;
		let decoder = codecs.make(&default_track.codec_params, &Default::default())?;
		Ok(Self {
			format_reader,
			decoder,
		})
	}
}

impl SoundData for StreamingSoundData {
	type Error = Error;

	type Handle = ();

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
		let (error_producer, error_consumer) = RingBuffer::new(ERROR_BUFFER_CAPACITY).split();
		Ok((Box::new(StreamingSound::new(self, error_producer)?), ()))
	}
}
