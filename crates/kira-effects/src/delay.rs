//! Adds echoes to a sound.

use kira::{
	dsp::{interpolate_frame, Frame},
	parameter::Parameters,
	track::Effect,
	value::{CachedValue, Value},
};

/// Settings for a [`Delay`] effect.
#[non_exhaustive]
pub struct DelaySettings {
	/// The delay time (in seconds).
	pub delay_time: Value,
	/// The amount of feedback.
	pub feedback: Value,
	/// The amount of audio the delay can store (in seconds).
	/// This affects the maximum delay time.
	pub buffer_length: f64,
	/// Effects that should be applied in the feedback loop.
	pub feedback_effects: Vec<Box<dyn Effect>>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: Value,
}

impl DelaySettings {
	/// Creates a new `DelaySettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the delay time (in seconds).
	pub fn delay_time(self, delay_time: impl Into<Value>) -> Self {
		Self {
			delay_time: delay_time.into(),
			..self
		}
	}

	/// Sets the amount of feedback.
	pub fn feedback(self, feedback: impl Into<Value>) -> Self {
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

	/// Adds an effect to the feedback loop.
	pub fn with_feedback_effect(mut self, effect: impl Effect + 'static) -> Self {
		self.feedback_effects.push(Box::new(effect));
		self
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: impl Into<Value>) -> Self {
		Self {
			mix: mix.into(),
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
			feedback_effects: vec![],
			mix: Value::Fixed(0.5),
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
pub struct Delay {
	delay_time: CachedValue,
	feedback: CachedValue,
	mix: CachedValue,
	state: DelayState,
	feedback_effects: Vec<Box<dyn Effect>>,
}

impl Delay {
	/// Creates a new delay effect.
	pub fn new(settings: DelaySettings) -> Self {
		Self {
			delay_time: CachedValue::new(0.0.., settings.delay_time, 0.5),
			feedback: CachedValue::new(-1.0..=1.0, settings.feedback, 0.5),
			mix: CachedValue::new(0.0..=1.0, settings.mix, 0.5),
			state: DelayState::Uninitialized {
				buffer_length: settings.buffer_length,
			},
			feedback_effects: settings.feedback_effects,
		}
	}
}

impl Effect for Delay {
	fn init(&mut self, sample_rate: u32) {
		if let DelayState::Uninitialized { buffer_length } = &self.state {
			self.state = DelayState::Initialized {
				buffer: vec![Frame::ZERO; (buffer_length * sample_rate as f64) as usize],
				write_position: 0,
			};
			for effect in &mut self.feedback_effects {
				effect.init(sample_rate);
			}
		} else {
			panic!("The delay should be in the uninitialized state before init")
		}
	}

	fn process(&mut self, input: Frame, dt: f64, parameters: &Parameters) -> Frame {
		if let DelayState::Initialized {
			buffer,
			write_position,
		} = &mut self.state
		{
			// update cached values
			self.delay_time.update(parameters);
			self.feedback.update(parameters);
			self.mix.update(parameters);

			// get the read position (in samples)
			let mut read_position = *write_position as f32 - (self.delay_time.get() / dt) as f32;
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
			let mut output = interpolate_frame(
				buffer[previous_sample_index],
				buffer[current_sample_index],
				buffer[next_sample_index],
				buffer[next_sample_index_2],
				fraction,
			);
			for effect in &mut self.feedback_effects {
				output = effect.process(output, dt, parameters);
			}

			// write output audio to the buffer
			*write_position += 1;
			*write_position %= buffer.len();
			buffer[*write_position] = input + output * self.feedback.get() as f32;

			let mix = self.mix.get() as f32;
			output * mix.sqrt() + input * (1.0 - mix).sqrt()
		} else {
			panic!("The delay should be initialized by the first process call")
		}
	}
}
