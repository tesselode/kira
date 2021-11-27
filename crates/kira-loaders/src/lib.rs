mod streaming;

use kira::{
	dsp::Frame,
	sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
pub use streaming::*;
use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	conv::{FromSample, IntoSample},
	io::MediaSourceStream,
	sample::Sample,
};

use std::{fmt::Display, fs::File, path::Path, sync::Arc};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	NoDefaultTrack,
	UnknownSampleRate,
	UnsupportedChannelConfiguration,
	IoError(std::io::Error),
	SymphoniaError(symphonia::core::errors::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoDefaultTrack => f.write_str("Could not determine the default audio track"),
			Error::UnknownSampleRate => {
				f.write_str("Could not detect the sample rate of the audio")
			}
			Error::UnsupportedChannelConfiguration => {
				f.write_str("Only mono and stereo audio is supported")
			}
			Error::IoError(error) => error.fmt(f),
			Error::SymphoniaError(error) => error.fmt(f),
		}
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::IoError(error) => Some(error),
			Error::SymphoniaError(error) => Some(error),
			_ => None,
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(v: std::io::Error) -> Self {
		Self::IoError(v)
	}
}

impl From<symphonia::core::errors::Error> for Error {
	fn from(v: symphonia::core::errors::Error) -> Self {
		Self::SymphoniaError(v)
	}
}

pub fn load(
	path: impl AsRef<Path>,
	settings: StaticSoundSettings,
) -> Result<StaticSoundData, Error> {
	let codecs = symphonia::default::get_codecs();
	let probe = symphonia::default::get_probe();
	let file = File::open(path)?;
	let mss = MediaSourceStream::new(Box::new(file), Default::default());
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
		.ok_or(Error::NoDefaultTrack)?
		.codec_params;
	let sample_rate = codec_params.sample_rate.ok_or(Error::UnknownSampleRate)?;
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
	Ok(StaticSoundData {
		sample_rate,
		frames: Arc::new(frames),
		settings,
	})
}

pub fn stream(
	path: impl AsRef<Path>,
	settings: StreamingSoundSettings,
) -> Result<StreamingSoundData, Error> {
	StreamingSoundData::new(path, settings)
}

fn load_frames_from_buffer_ref(
	frames: &mut Vec<Frame>,
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
	frames: &mut Vec<Frame>,
	buffer: &AudioBuffer<S>,
) -> Result<(), Error>
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
		_ => return Err(Error::UnsupportedChannelConfiguration),
	}
	Ok(())
}
