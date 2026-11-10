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

#[cfg(test)]
mod test;

/// The number of frames to decode before the frame that a seek is
/// targeting. This is more than a packet of AAC (1024 frames) or MP3 (1152
/// frames), so a whole packet is decoded to prepare the decoder.
const SEEK_PREROLL_FRAMES: usize = 2048;

pub(crate) struct SymphoniaDecoder {
	format_reader: Box<dyn FormatReader>,
	decoder: Box<dyn AudioDecoder>,
	sample_rate: u32,
	num_frames: usize,
	track_id: u32,
	/// The frames from the target of the last seek to the end of the packet
	/// they were decoded from, which the next call to `decode` returns first.
	frames_after_seek: Vec<Frame>,
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
			frames_after_seek: vec![],
		})
	}

	/// Decodes the next packet of the track. Returns the index of the first
	/// frame of the packet and the decoded frames, or `None` at the end of
	/// the file.
	///
	/// The index comes from the packet because the first packet decoded after
	/// a reset can come out empty (Ogg Vorbis), though the file counts frames
	/// for it.
	fn decode_packet(&mut self) -> Result<Option<(usize, Vec<Frame>)>, FromFileError> {
		let packet = loop {
			match self.format_reader.next_packet()? {
				Some(packet) if packet.track_id == self.track_id => break packet,
				Some(_) => continue,
				None => return Ok(None),
			}
		};
		let first_frame_index = packet.pts.saturating_add(packet.trim_start).get();
		let buffer = self.decoder.decode(&packet)?;
		Ok(Some((
			first_frame_index.max(0) as usize,
			load_frames_from_buffer_ref(&buffer)?,
		)))
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
		if !self.frames_after_seek.is_empty() {
			return Ok(std::mem::take(&mut self.frames_after_seek));
		}
		Ok(self
			.decode_packet()?
			.map(|(_, frames)| frames)
			.unwrap_or_default())
	}

	fn seek(&mut self, index: usize) -> Result<usize, Self::Error> {
		self.frames_after_seek.clear();
		// Codecs like AAC, MP3, and Vorbis decode a frame using the frames
		// before it, so decoding right from the target gives wrong frames at
		// first. Seek earlier, reset the decoder, and decode up to the target
		// the way Symphonia leaves to the caller.
		self.format_reader.seek(
			SeekMode::Accurate,
			SeekTo::Timestamp {
				ts: Timestamp::new(index.saturating_sub(SEEK_PREROLL_FRAMES) as i64),
				track_id: self.track_id,
			},
		)?;
		self.decoder.reset();
		while let Some((packet_index, mut frames)) = self.decode_packet()? {
			if !frames.is_empty() && packet_index + frames.len() > index {
				self.frames_after_seek = frames.split_off(index.saturating_sub(packet_index));
				return Ok(packet_index.max(index));
			}
		}
		Ok(index.min(self.num_frames))
	}
}
