use crate::{parameter::Parameters, util, CachedValue, Frame, Value};

use super::Effect;

/// Settings for a [`Delay`] effect.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct DelaySettings {
	/// The delay time (in seconds).
	delay_time: Value<f64>,
	/// The amount of feedback.
	feedback: Value<f64>,
	/// The number of frames of audio the delay will store.
	/// This affects the maximum delay time.
	buffer_length: usize,
}

impl DelaySettings {
	/// Creates a new `DelaySettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the delay time (in seconds).
	pub fn delay_time(self, delay_time: impl Into<Value<f64>>) -> Self {
		Self {
			delay_time: delay_time.into(),
			..self
		}
	}

	/// Sets the amount of feedback.
	pub fn feedback(self, feedback: impl Into<Value<f64>>) -> Self {
		Self {
			feedback: feedback.into(),
			..self
		}
	}

	/// Sets the number of frames of audio the delay will store.
	pub fn buffer_length(self, buffer_length: usize) -> Self {
		Self {
			buffer_length,
			..self
		}
	}
}

impl Default for DelaySettings {
	fn default() -> Self {
		Self {
			delay_time: Value::Fixed(0.5),
			feedback: Value::Fixed(0.5),
			buffer_length: 48000 * 10,
		}
	}
}

/// An effect that repeats audio after a certain delay. Useful
/// for creating echo effects.
#[derive(Debug, Clone)]
pub struct Delay {
	delay_time: CachedValue<f64>,
	feedback: CachedValue<f64>,
	buffer: Vec<Frame>,
	write_position: usize,
}

impl Delay {
	/// Creates a new delay effect.
	pub fn new(settings: DelaySettings) -> Self {
		Self {
			delay_time: CachedValue::new(settings.delay_time, 0.5),
			feedback: CachedValue::new(settings.feedback, 0.5),
			buffer: vec![Frame::from_mono(0.0); settings.buffer_length],
			write_position: 0,
		}
	}
}

impl Effect for Delay {
	fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame {
		// update cached values
		self.delay_time.update(parameters);
		self.feedback.update(parameters);

		// write input audio to the buffer
		self.write_position += 1;
		self.write_position %= self.buffer.len();
		self.buffer[self.write_position] = input;

		// get the read position (in samples)
		let mut read_position = self.write_position as f32 - (self.delay_time.value() / dt) as f32;
		while read_position < 0.0 {
			read_position += self.buffer.len() as f32;
		}

		// read an interpolated sample
		let current_sample_index = read_position as usize;
		let previous_sample_index = if current_sample_index == 0 {
			self.buffer.len() - 2
		} else {
			current_sample_index - 1
		};
		let next_sample_index = (current_sample_index + 1) % self.buffer.len();
		let next_sample_index_2 = (current_sample_index + 2) % self.buffer.len();
		let fraction = read_position % 1.0;
		util::interpolate_frame(
			self.buffer[previous_sample_index],
			self.buffer[current_sample_index],
			self.buffer[next_sample_index],
			self.buffer[next_sample_index_2],
			fraction,
		)
	}
}
