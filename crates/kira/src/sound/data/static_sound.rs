use std::{fmt::Display, io::Cursor};

use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	conv::{FromSample, IntoSample},
	io::{MediaSource, MediaSourceStream},
	sample::Sample,
};

use crate::dsp::Frame;

use super::SoundData;

#[derive(Clone, PartialEq)]
pub struct StaticSoundData {
	pub sample_rate: u32,
	pub frames: Vec<Frame>,
}

impl StaticSoundData {
	fn from_media_source(media_source: Box<dyn MediaSource>) -> Result<Self, FromFileError> {
		let codecs = symphonia::default::get_codecs();
		let probe = symphonia::default::get_probe();
		let mss = MediaSourceStream::new(media_source, Default::default());
		let mut format_reader = probe
			.format(
				&Default::default(),
				mss,
				&Default::default(),
				&Default::default(),
			)?
			.format;
		let codec_params = &format_reader
			.default_track()
			.ok_or(FromFileError::NoDefaultTrack)?
			.codec_params;
		let sample_rate = codec_params
			.sample_rate
			.ok_or(FromFileError::UnknownSampleRate)?;
		let mut decoder = codecs.make(codec_params, &Default::default())?;
		let mut frames = vec![];
		loop {
			match format_reader.next_packet() {
				Ok(packet) => {
					let buffer = decoder.decode(&packet)?;
					load_frames_from_buffer_ref(&mut frames, &buffer)?;
				}
				Err(error) => match error {
					symphonia::core::errors::Error::IoError(error) => {
						if error.kind() == std::io::ErrorKind::UnexpectedEof {
							break;
						}
						return Err(symphonia::core::errors::Error::IoError(error).into());
					}
					error => return Err(error.into()),
				},
			}
		}
		Ok(Self {
			sample_rate,
			frames,
		})
	}

	/// Loads an audio file into a [`StaticSoundData`].
	pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, FromFileError> {
		Self::from_media_source(Box::new(std::fs::File::open(path)?))
	}

	/// Loads a cursor wrapping audio file data into a [`StaticSoundData`].
	pub fn from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
		cursor: Cursor<T>,
	) -> Result<Self, FromFileError> {
		Self::from_media_source(Box::new(cursor))
	}
}

impl SoundData for StaticSoundData {
	fn sample_rate(&mut self) -> u32 {
		self.sample_rate
	}

	fn len(&mut self) -> usize {
		self.frames.len()
	}

	fn frame(&mut self, index: usize) -> Frame {
		self.frames[index]
	}
}

/// Errors that can occur when loading or streaming an audio file.
#[derive(Debug)]
#[non_exhaustive]
pub enum FromFileError {
	/// Could not determine the default audio track in the file.
	NoDefaultTrack,
	/// Could not determine the sample rate of the audio.
	UnknownSampleRate,
	/// The audio uses an unsupported channel configuration. Only
	/// mono and stereo audio is supported.
	UnsupportedChannelConfiguration,
	/// An error occurred while reading the file from the filesystem.
	IoError(std::io::Error),
	/// An error occurred when parsing the file.
	SymphoniaError(symphonia::core::errors::Error),
}

impl Display for FromFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FromFileError::NoDefaultTrack => {
				f.write_str("Could not determine the default audio track")
			}
			FromFileError::UnknownSampleRate => {
				f.write_str("Could not detect the sample rate of the audio")
			}
			FromFileError::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			FromFileError::IoError(error) => error.fmt(f),
			FromFileError::SymphoniaError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for FromFileError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			FromFileError::IoError(error) => Some(error),
			FromFileError::SymphoniaError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<std::io::Error> for FromFileError {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<symphonia::core::errors::Error> for FromFileError {
	fn from(v: symphonia::core::errors::Error) -> Self {
		Self::SymphoniaError(v)
	}
}

fn load_frames_from_buffer_ref(
	frames: &mut Vec<Frame>,
	buffer: &AudioBufferRef,
) -> Result<(), FromFileError> {
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
	frames: &mut Vec<Frame>,
	buffer: &AudioBuffer<S>,
) -> Result<(), FromFileError>
where
	f32: FromSample<S>,
{
	match buffer.spec().channels.count() {
		1 => {
			for sample in buffer.chan(0) {
				frames.push(Frame::from_mono((*sample).into_sample()));
			}
		}
		2 => {
			for (left, right) in buffer.chan(0).iter().zip(buffer.chan(1).iter()) {
				frames.push(Frame::new((*left).into_sample(), (*right).into_sample()));
			}
		}
		_ => return Err(FromFileError::UnsupportedChannelConfiguration),
	}
	Ok(())
}
