use std::convert::TryInto;

use crate::{
	frame::Frame,
	sound::{FromFileError, symphonia::load_frames_from_buffer_ref},
};
use symphonia::core::{
	codecs::{CodecParameters, audio::AudioDecoder},
	formats::{FormatReader, SeekMode, SeekTo, TrackType, probe::Hint},
	io::{MediaSource, MediaSourceStream},
	units::Timestamp,
};

pub(crate) struct SymphoniaDecoder {
	format_reader: Box<dyn FormatReader>,
	decoder: Box<dyn AudioDecoder>,
	sample_rate: u32,
	num_frames: usize,
	track_id: u32,
}

#[cfg(all(test, feature = "ogg", feature = "vorbis"))]
mod tests {
	use super::*;
	use crate::sound::streaming::Decoder;

	#[test]
	fn decodes_vorbis_through_eof_and_seeks_back() {
		let file = std::fs::File::open(concat!(
			env!("CARGO_MANIFEST_DIR"),
			"/../examples/assets/blip.ogg"
		))
		.unwrap();
		let mut decoder = SymphoniaDecoder::new(Box::new(file)).unwrap();
		let mut decoded_frames = 0;
		loop {
			let frames = decoder.decode().unwrap();
			if frames.is_empty() {
				break;
			}
			decoded_frames += frames.len();
			assert!(decoded_frames <= decoder.num_frames());
		}
		assert!(decoded_frames > 0);
		assert!(decoder.decode().unwrap().is_empty());
		decoder.seek(0).unwrap();
		assert!(!decoder.decode().unwrap().is_empty());
	}
}

impl SymphoniaDecoder {
	pub(crate) fn new(media_source: Box<dyn MediaSource>) -> Result<Self, FromFileError> {
		let codecs = symphonia::default::get_codecs();
		let probe = symphonia::default::get_probe();
		let mss = MediaSourceStream::new(media_source, Default::default());
		let format_reader = probe.probe(
			&Hint::default(),
			mss,
			Default::default(),
			Default::default(),
		)?;
		let default_track = format_reader
			.default_track(TrackType::Audio)
			.ok_or(FromFileError::NoDefaultTrack)?;
		let audio_params = match default_track.codec_params.as_ref() {
			Some(CodecParameters::Audio(p)) => p,
			_ => return Err(FromFileError::NoDefaultTrack),
		};
		let sample_rate = audio_params
			.sample_rate
			.ok_or(FromFileError::UnknownSampleRate)?;
		let num_frames = default_track
			.num_frames
			.ok_or(FromFileError::UnknownSampleRate)?
			.try_into()
			.expect("could not convert u64 into usize");
		let decoder = codecs.make_audio_decoder(audio_params, &Default::default())?;
		let track_id = default_track.id;
		Ok(Self {
			format_reader,
			decoder,
			sample_rate,
			num_frames,
			track_id,
		})
	}
}

impl super::Decoder for SymphoniaDecoder {
	type Error = FromFileError;

	fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	fn num_frames(&self) -> usize {
		self.num_frames
	}

	fn decode(&mut self) -> Result<Vec<Frame>, Self::Error> {
		loop {
			let packet = loop {
				match self.format_reader.next_packet()? {
					Some(packet) if packet.track_id == self.track_id => break packet,
					Some(_) => continue,
					None => return Ok(vec![]),
				}
			};
			let buffer = self.decoder.decode(&packet)?;
			let frames = load_frames_from_buffer_ref(&buffer)?;
			if !frames.is_empty() {
				return Ok(frames);
			}
		}
	}

	fn seek(&mut self, index: usize) -> Result<usize, Self::Error> {
		let seeked_to = self.format_reader.seek(
			SeekMode::Accurate,
			SeekTo::Timestamp {
				ts: Timestamp::new(index as i64),
				track_id: self.track_id,
			},
		)?;
		let actual = seeked_to.actual_ts.get();
		Ok(if actual < 0 {
			0
		} else {
			(actual as usize).min(self.num_frames)
		})
	}
}
