use symphonia::core::audio::{
	Audio, AudioBuffer, GenericAudioBufferRef,
	conv::{FromSample, IntoSample},
	sample::Sample,
};

use crate::frame::Frame;

use super::FromFileError;

pub fn load_frames_from_buffer_ref(
	buffer: &GenericAudioBufferRef,
) -> Result<Vec<Frame>, FromFileError> {
	match buffer {
		GenericAudioBufferRef::U8(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::U16(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::U24(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::U32(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::S8(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::S16(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::S24(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::S32(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::F32(buffer) => load_frames_from_buffer(buffer),
		GenericAudioBufferRef::F64(buffer) => load_frames_from_buffer(buffer),
	}
}

pub fn load_frames_from_buffer<S: Sample>(
	buffer: &AudioBuffer<S>,
) -> Result<Vec<Frame>, FromFileError>
where
	f32: FromSample<S>,
{
	match buffer.num_planes() {
		1 => Ok(buffer
			.plane(0)
			.unwrap()
			.iter()
			.map(|sample| Frame::from_mono((*sample).into_sample()))
			.collect()),
		2 => Ok(buffer
			.plane(0)
			.unwrap()
			.iter()
			.zip(buffer.plane(1).unwrap().iter())
			.map(|(left, right)| Frame::new((*left).into_sample(), (*right).into_sample()))
			.collect()),
		_ => Err(FromFileError::UnsupportedChannelConfiguration),
	}
}
