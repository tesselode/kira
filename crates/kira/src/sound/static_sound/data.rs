use std::{sync::Arc, time::Duration};

use ringbuf::RingBuffer;

use crate::{
	dsp::{interpolate_frame, Frame, I24},
	sound::{Sound, SoundData},
};

use super::{handle::StaticSoundHandle, sound::StaticSound, StaticSoundSettings};

const COMMAND_BUFFER_CAPACITY: usize = 8;

#[derive(Clone)]
pub enum Samples {
	I16Mono(Vec<i16>),
	I16Stereo(Vec<[i16; 2]>),
	I24Mono(Vec<I24>),
	I24Stereo(Vec<[I24; 2]>),
	F32Mono(Vec<f32>),
	F32Stereo(Vec<[f32; 2]>),
}

impl Samples {
	fn len(&self) -> usize {
		match self {
			Samples::I16Mono(samples) => samples.len(),
			Samples::I16Stereo(samples) => samples.len(),
			Samples::I24Mono(samples) => samples.len(),
			Samples::I24Stereo(samples) => samples.len(),
			Samples::F32Mono(samples) => samples.len(),
			Samples::F32Stereo(samples) => samples.len(),
		}
	}
}

#[derive(Clone)]
pub struct StaticSoundData {
	pub sample_rate: u32,
	pub samples: Arc<Samples>,
	pub settings: StaticSoundSettings,
}

impl StaticSoundData {
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.samples.len() as f64 / self.sample_rate as f64)
	}

	fn frame_at_index(&self, index: usize) -> Frame {
		match self.samples.as_ref() {
			Samples::I16Mono(samples) => samples
				.get(index)
				.copied()
				.map(|sample| sample.into())
				.unwrap_or(Frame::ZERO),
			Samples::I16Stereo(samples) => samples
				.get(index)
				.copied()
				.map(|sample| sample.into())
				.unwrap_or(Frame::ZERO),
			Samples::I24Mono(samples) => samples
				.get(index)
				.copied()
				.map(|sample| sample.into())
				.unwrap_or(Frame::ZERO),
			Samples::I24Stereo(samples) => samples
				.get(index)
				.copied()
				.map(|sample| sample.into())
				.unwrap_or(Frame::ZERO),
			Samples::F32Mono(samples) => samples
				.get(index)
				.copied()
				.map(|sample| sample.into())
				.unwrap_or(Frame::ZERO),
			Samples::F32Stereo(samples) => samples
				.get(index)
				.copied()
				.map(|sample| sample.into())
				.unwrap_or(Frame::ZERO),
		}
	}

	pub fn frame_at_position(&self, position: f64) -> Frame {
		let sample_position = self.sample_rate as f64 * position;
		let fraction = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let previous = if current_sample_index == 0 {
			Frame::ZERO
		} else {
			self.frame_at_index(current_sample_index - 1)
		};
		let current = self.frame_at_index(current_sample_index);
		let next_1 = self.frame_at_index(current_sample_index + 1);
		let next_2 = self.frame_at_index(current_sample_index + 2);
		interpolate_frame(previous, current, next_1, next_2, fraction)
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = StaticSoundHandle;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let sound = StaticSound::new(self, command_consumer);
		let shared = sound.shared();
		Ok((
			Box::new(sound),
			StaticSoundHandle {
				command_producer,
				shared,
			},
		))
	}
}
