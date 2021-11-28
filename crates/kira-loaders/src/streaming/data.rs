use std::{fs::File, path::Path};

use kira::sound::SoundData;
use ringbuf::RingBuffer;
use symphonia::core::{codecs::Decoder, formats::FormatReader, io::MediaSourceStream, probe::Hint};

use crate::{Error, StreamingSoundHandle, StreamingSoundSettings};

use super::sound::StreamingSound;

const COMMAND_BUFFER_CAPACITY: usize = 8;
const ERROR_BUFFER_CAPACITY: usize = 8;

/// A streaming sound that is not playing yet.
pub struct StreamingSoundData {
	pub(crate) format_reader: Box<dyn FormatReader>,
	pub(crate) decoder: Box<dyn Decoder>,
	pub(crate) sample_rate: u32,
	pub(crate) track_id: u32,
	/// Settings for the streaming sound.
	pub settings: StreamingSoundSettings,
}

impl StreamingSoundData {
	pub(crate) fn new(
		path: impl AsRef<Path>,
		settings: StreamingSoundSettings,
	) -> Result<Self, Error> {
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
		let sample_rate = default_track
			.codec_params
			.sample_rate
			.ok_or(Error::UnknownSampleRate)?;
		let decoder = codecs.make(&default_track.codec_params, &Default::default())?;
		let track_id = default_track.id;
		Ok(Self {
			format_reader,
			decoder,
			sample_rate,
			track_id,
			settings,
		})
	}
}

impl SoundData for StreamingSoundData {
	type Error = Error;

	type Handle = StreamingSoundHandle;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn kira::sound::Sound>, Self::Handle), Self::Error> {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let (error_producer, error_consumer) = RingBuffer::new(ERROR_BUFFER_CAPACITY).split();
		let sound = StreamingSound::new(self, command_consumer, error_producer)?;
		let shared = sound.shared();
		Ok((
			Box::new(sound),
			StreamingSoundHandle {
				shared,
				command_producer,
				error_consumer,
			},
		))
	}
}
