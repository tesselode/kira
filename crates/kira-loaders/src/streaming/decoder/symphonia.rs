use std::collections::VecDeque;

use kira::dsp::Frame;
use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	codecs::Decoder,
	conv::{FromSample, IntoSample},
	formats::{FormatReader, SeekMode, SeekTo},
	io::{MediaSource, MediaSourceStream},
	probe::Hint,
	sample::Sample,
};

use crate::Error;

pub(crate) struct SymphoniaDecoder {
	format_reader: Box<dyn FormatReader>,
	decoder: Box<dyn Decoder>,
	sample_rate: u32,
	track_id: u32,
}

impl SymphoniaDecoder {
	pub(crate) fn new(media_source: Box<dyn MediaSource>) -> Result<Self, Error> {
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
		})
	}
}

impl super::Decoder for SymphoniaDecoder {
	type Error = Error;

	fn sample_rate(&self) -> u32 {
		self.sample_rate
	}

	fn decode(&mut self, frames: &mut VecDeque<Frame>) -> Result<bool, Self::Error> {
		match self.format_reader.next_packet() {
			Ok(packet) => {
				let buffer = self.decoder.decode(&packet)?;
				load_frames_from_buffer_ref(frames, &buffer)?;
			}
			Err(error) => match error {
				symphonia::core::errors::Error::IoError(error) => {
					if error.kind() == std::io::ErrorKind::UnexpectedEof {
						return Ok(true);
					}
					return Err(symphonia::core::errors::Error::IoError(error).into());
				}
				error => return Err(error.into()),
			},
		}
		Ok(false)
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

fn load_frames_from_buffer_ref(
	frames: &mut VecDeque<Frame>,
	buffer: &AudioBufferRef,
) -> Result<(), Error> {
	match buffer {
		AudioBufferRef::U8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F64(buffer) => load_frames_from_buffer(frames, buffer),
	}
}

fn load_frames_from_buffer<S: Sample>(
	frames: &mut VecDeque<Frame>,
	buffer: &AudioBuffer<S>,
) -> Result<(), Error>
where
	f32: FromSample<S>,
{
	match buffer.spec().channels.count() {
		1 => {
			for sample in buffer.chan(0) {
				frames.push_back(Frame::from_mono((*sample).into_sample()));
			}
		}
		2 => {
			for (left, right) in buffer.chan(0).iter().zip(buffer.chan(1).iter()) {
				frames.push_back(Frame::new((*left).into_sample(), (*right).into_sample()));
			}
		}
		_ => return Err(Error::UnsupportedChannelConfiguration),
	}
	Ok(())
}
