use std::time::Duration;

use crate::{
	Decibels, Mix, Value,
	effect::{Effect, EffectBuilder},
};

use super::{Delay, DelayHandle, command_writers_and_readers};

/// Configures a delay effect.
pub struct DelayBuilder {
	/// The amount of time the input audio is delayed by.
	pub(super) delay_time: Duration,
	/// The amount of feedback.
	pub(super) feedback: Value<Decibels>,
	/// Effects that should be applied in the feedback loop.
	pub(super) feedback_effects: Vec<Box<dyn Effect>>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal.
	pub(super) mix: Value<Mix>,
}

impl DelayBuilder {
	/// Creates a new [`DelayBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the amount of time the input audio is delayed by.
	#[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
	pub fn delay_time(self, delay_time: Duration) -> Self {
		Self { delay_time, ..self }
	}

	/// Sets the amount of feedback.
	#[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
	pub fn feedback(self, feedback: impl Into<Value<Decibels>>) -> Self {
		Self {
			feedback: feedback.into(),
			..self
		}
	}

	/// Adds an effect to the feedback loop.
	pub fn add_feedback_effect<B: EffectBuilder>(&mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.feedback_effects.push(effect);
		handle
	}

	/// Adds an effect to the feedback loop and returns the [`DelayBuilder`].
	///
	/// If you need a handle to the newly added effect, use [`DelayBuilder::add_feedback_effect`].
	pub fn with_feedback_effect<B: EffectBuilder>(mut self, builder: B) -> Self {
		self.add_feedback_effect(builder);
		self
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	#[must_use = "This method consumes self and returns a modified DelayBuilder, so the return value should be used"]
	pub fn mix(self, mix: impl Into<Value<Mix>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for DelayBuilder {
	fn default() -> Self {
		Self {
			delay_time: Duration::from_millis(500),
			feedback: Value::Fixed(Decibels(-6.0)),
			feedback_effects: vec![],
			mix: Value::Fixed(Mix(0.5)),
		}
	}
}

impl EffectBuilder for DelayBuilder {
	type Handle = DelayHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(Delay::new(self, command_readers)),
			DelayHandle { command_writers },
		)
	}
}
