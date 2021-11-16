use std::{sync::Arc, time::Duration};

use ringbuf::RingBuffer;

use crate::{
	dsp::{interpolate_frame, Frame, I24},
	sound::{Sound, SoundData},
};

use super::{handle::StaticSoundHandle, sound::StaticSound, StaticSoundSettings};

const COMMAND_BUFFER_CAPACITY: usize = 8;

/// A buffer of audio samples in various formats.
#[derive(Clone)]
pub enum Samples {
	/// 16-bit mono audio stored with signed integers.
	I16Mono(Vec<i16>),
	/// 16-bit stereo audio stored with signed integers.
	I16Stereo(Vec<[i16; 2]>),
	/// 24-bit mono audio stored with signed integers.
	I24Mono(Vec<I24>),
	/// 24-bit stereo audio stored with signed integers.
	I24Stereo(Vec<[I24; 2]>),
	/// Mono audio stored wtih 32-bit floating point numbers.
	F32Mono(Vec<f32>),
	/// Stereo audio stored wtih 32-bit floating point numbers.
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

/// A piece of audio loaded into memory all at once.
///
/// These can be cheaply cloned, as the audio data is shared
/// among all clones.
#[derive(Clone)]
pub struct StaticSoundData {
	/// The sample rate of the audio (in Hz).
	pub sample_rate: u32,
	/// The raw samples that make up the audio.
	pub samples: Arc<Samples>,
	/// Settings for the sound.
	pub settings: StaticSoundSettings,
}

impl StaticSoundData {
	/// Returns the duration of the audio.
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

	/// Gets the [`Frame`] at an arbitrary time in seconds.
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
