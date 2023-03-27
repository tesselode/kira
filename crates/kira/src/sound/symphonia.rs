use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	conv::{FromSample, IntoSample},
	sample::Sample,
};

use crate::dsp::Frame;

use super::FromFileError;

pub fn load_frames_from_buffer_ref(buffer: &AudioBufferRef) -> Result<Vec<Frame>, FromFileError> {
	match buffer {
		AudioBufferRef::U8(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::U16(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::U24(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::U32(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::S8(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::S16(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::S24(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::S32(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::F32(buffer) => load_frames_from_buffer(buffer),
		AudioBufferRef::F64(buffer) => load_frames_from_buffer(buffer),
	}
}

pub fn load_frames_from_buffer<S: Sample>(
	buffer: &AudioBuffer<S>,
) -> Result<Vec<Frame>, FromFileError>
where
	f32: FromSample<S>,
{
	match buffer.spec().channels.count() {
		1 => Ok(buffer
			.chan(0)
			.iter()
			.map(|sample| Frame::from_mono((*sample).into_sample()))
			.collect()),
		2 => Ok(buffer
			.chan(0)
			.iter()
			.zip(buffer.chan(1).iter())
			.map(|(left, right)| Frame::new((*left).into_sample(), (*right).into_sample()))
			.collect()),
		_ => Err(FromFileError::UnsupportedChannelConfiguration),
	}
}
