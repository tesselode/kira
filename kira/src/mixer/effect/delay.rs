//! Adds echoes to a sound.

use crate::{parameter::Parameters, util, CachedValue, Frame, Value};

use super::{
	filter::{Filter, FilterSettings},
	Effect,
};

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
	/// The amount of audio the delay can store.
	/// This affects the maximum delay time.
	buffer_length: f64,
	/// Whether a filter should be added to the feedback loop,
	/// and if so, the settings to use for the filter.
	filter_settings: Option<FilterSettings>,
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

	/// Sets the amount of audio the delay can store.
	pub fn buffer_length(self, buffer_length: f64) -> Self {
		Self {
			buffer_length,
			..self
		}
	}

	/// Sets whether a filter should be added to the feedback loop,
	/// and if so, the settings to use for the filter.
	pub fn filter_settings(self, filter_settings: impl Into<Option<FilterSettings>>) -> Self {
		Self {
			filter_settings: filter_settings.into(),
			..self
		}
	}
}

impl Default for DelaySettings {
	fn default() -> Self {
		Self {
			delay_time: Value::Fixed(0.5),
			feedback: Value::Fixed(0.5),
			buffer_length: 10.0,
			filter_settings: None,
		}
	}
}

#[derive(Debug, Clone)]
enum DelayState {
	Uninitialized {
		buffer_length: f64,
	},
	Initialized {
		buffer: Vec<Frame>,
		write_position: usize,
	},
}

/// An effect that repeats audio after a certain delay. Useful
/// for creating echo effects.
#[derive(Debug, Clone)]
pub struct Delay {
	delay_time: CachedValue<f64>,
	feedback: CachedValue<f64>,
	state: DelayState,
	filter: Option<Filter>,
}

impl Delay {
	/// Creates a new delay effect.
	pub fn new(settings: DelaySettings) -> Self {
		Self {
			delay_time: CachedValue::new(settings.delay_time, 0.5),
			feedback: CachedValue::new(settings.feedback, 0.5),
			state: DelayState::Uninitialized {
				buffer_length: settings.buffer_length,
			},
			filter: settings
				.filter_settings
				.map(|settings| Filter::new(settings)),
		}
	}
}

impl Effect for Delay {
	fn init(&mut self, sample_rate: u32) {
		if let DelayState::Uninitialized { buffer_length } = &self.state {
			self.state = DelayState::Initialized {
				buffer: vec![Frame::from_mono(0.0); (buffer_length * sample_rate as f64) as usize],
				write_position: 0,
			}
		} else {
			panic!("The delay should be in the uninitialized state before init")
		}
	}

	fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame {
		if let DelayState::Initialized {
			buffer,
			write_position,
		} = &mut self.state
		{
			// update cached values
			self.delay_time.update(parameters);
			self.feedback.update(parameters);

			// get the read position (in samples)
			let mut read_position = *write_position as f32 - (self.delay_time.value() / dt) as f32;
			while read_position < 0.0 {
				read_position += buffer.len() as f32;
			}

			// read an interpolated sample
			let current_sample_index = read_position as usize;
			let previous_sample_index = if current_sample_index == 0 {
				buffer.len() - 2
			} else {
				current_sample_index - 1
			};
			let next_sample_index = (current_sample_index + 1) % buffer.len();
			let next_sample_index_2 = (current_sample_index + 2) % buffer.len();
			let fraction = read_position % 1.0;
			let output = util::interpolate_frame(
				buffer[previous_sample_index],
				buffer[current_sample_index],
				buffer[next_sample_index],
				buffer[next_sample_index_2],
				fraction,
			);

			// write input audio to the buffer
			*write_position += 1;
			*write_position %= buffer.len();
			let filtered_output = match &mut self.filter {
				Some(filter) => filter.process(dt, output, parameters),
				None => output,
			};
			buffer[*write_position] = input + filtered_output * self.feedback.value() as f32;

			filtered_output
		} else {
			panic!("The delay should be initialized by the first process call")
		}
	}
}
