use std::convert::TryInto;

use crate::{
	dsp::Frame,
	sound::{symphonia::load_frames_from_buffer_ref, FromFileError},
};
use symphonia::core::{
	codecs::Decoder,
	formats::{FormatReader, SeekMode, SeekTo},
	io::{MediaSource, MediaSourceStream},
	probe::Hint,
};

pub(crate) struct SymphoniaDecoder {
	format_reader: Box<dyn FormatReader>,
	decoder: Box<dyn Decoder>,
	sample_rate: u32,
	num_frames: usize,
	track_id: u32,
}

impl SymphoniaDecoder {
	#[must_use]
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
		let num_frames = default_track
			.codec_params
			.n_frames
			.ok_or(FromFileError::UnknownSampleRate)?
			.try_into()
			.expect("could not convert u64 into usize");
		let decoder = codecs.make(&default_track.codec_params, &Default::default())?;
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
		let packet = self.format_reader.next_packet()?;
		let buffer = self.decoder.decode(&packet)?;
		load_frames_from_buffer_ref(&buffer)
	}

	fn seek(&mut self, index: usize) -> Result<usize, Self::Error> {
		let seeked_to = self.format_reader.seek(
			SeekMode::Accurate,
			SeekTo::TimeStamp {
				ts: index.try_into().expect("could not convert usize into u64"),
				track_id: self.track_id,
			},
		)?;
		Ok(seeked_to
			.actual_ts
			.try_into()
			.expect("could not convert u64 into usize"))
	}
}
