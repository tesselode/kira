use ringbuf::HeapRb;

use crate::{
	track::effect::{Effect, EffectBuilder},
	tween::Value,
	Volume,
};

use super::{Delay, DelayHandle};

const COMMAND_CAPACITY: usize = 8;

/// Configures a delay effect.
#[non_exhaustive]
pub struct DelayBuilder {
	/// The delay time (in seconds).
	pub(super) delay_time: Value<f64>,
	/// The amount of feedback.
	pub(super) feedback: Value<Volume>,
	/// The amount of audio the delay can store (in seconds).
	/// This affects the maximum delay time.
	pub(super) buffer_length: f64,
	/// Effects that should be applied in the feedback loop.
	pub(super) feedback_effects: Vec<Box<dyn Effect>>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub(super) mix: Value<f64>,
}

impl DelayBuilder {
	/// Creates a new [`DelayBuilder`] with the default settings.
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
	pub fn feedback(self, feedback: impl Into<Value<Volume>>) -> Self {
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
	pub fn add_feedback_effect<B: EffectBuilder>(&mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.feedback_effects.push(effect);
		handle
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for DelayBuilder {
	fn default() -> Self {
		Self {
			delay_time: Value::Fixed(0.5),
			feedback: Value::Fixed(Volume::Amplitude(0.5)),
			buffer_length: 10.0,
			feedback_effects: vec![],
			mix: Value::Fixed(0.5),
		}
	}
}

impl EffectBuilder for DelayBuilder {
	type Handle = DelayHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		(
			Box::new(Delay::new(self, command_consumer)),
			DelayHandle { command_producer },
		)
	}
}
