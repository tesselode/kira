use crate::sound::{symphonia::load_frames_from_buffer_ref, FromFileError};
use symphonia::core::{
	codecs::Decoder,
	formats::{FormatReader, SeekMode, SeekTo},
	io::{MediaSource, MediaSourceStream},
	probe::Hint,
};

use super::DecodeResponse;

pub(crate) struct SymphoniaDecoder {
	format_reader: Box<dyn FormatReader>,
	decoder: Box<dyn Decoder>,
	sample_rate: u32,
	track_id: u32,
}

impl SymphoniaDecoder {
	pub(crate) fn new(media_source: Box<dyn MediaSource>) -> Result<Self, FromFileError> {
		let codecs = symphonia::default::get_codecs();
		let probe = symphonia::default::get_probe();
		let mss = MediaSourceStream::new(media_source, Default::default());
		let format_reader = probe
			.format(
				&Hint::default(),
				mss,
				&Default::default(),
				&Default::default(),
			)?
			.format;
		let default_track = format_reader
			.default_track()
			.ok_or(FromFileError::NoDefaultTrack)?;
		let sample_rate = default_track
			.codec_params
			.sample_rate
			.ok_or(FromFileError::UnknownSampleRate)?;
		let decoder = codecs.make(&default_track.codec_params, &Default::default())?;
		let track_id = default_track.id;
		Ok(Self {
			format_reader,
			decoder,
			sample_rate,
			track_id,
		})
	}
}

impl super::Decoder for SymphoniaDecoder {
	type Error = FromFileError;

	fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	fn decode(&mut self) -> Result<DecodeResponse, Self::Error> {
		match self.format_reader.next_packet() {
			Ok(packet) => {
				let buffer = self.decoder.decode(&packet)?;
				Ok(DecodeResponse::DecodedFrames(load_frames_from_buffer_ref(
					&buffer,
				)?))
			}
			Err(error) => match error {
				symphonia::core::errors::Error::IoError(error) => {
					if error.kind() == std::io::ErrorKind::UnexpectedEof {
						Ok(DecodeResponse::ReachedEndOfAudio)
					} else {
						Err(symphonia::core::errors::Error::IoError(error).into())
					}
				}
				error => Err(error.into()),
			},
		}
	}

	fn seek(&mut self, index: u64) -> Result<u64, Self::Error> {
		let seeked_to = self.format_reader.seek(
			SeekMode::Accurate,
			SeekTo::TimeStamp {
				ts: index,
				track_id: self.track_id,
			},
		)?;
		Ok(seeked_to.actual_ts)
	}
}
