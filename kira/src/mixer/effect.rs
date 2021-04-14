//! Modifies audio in real time.

pub mod delay;
pub mod distortion;
pub mod filter;
pub mod reverb;

use crate::{value::Value, Frame};

pub struct EffectSettings {
	pub(crate) enabled: bool,
	pub(crate) mix: Value<f64>,
}

impl EffectSettings {
	pub fn new() -> Self {
		Self {
			enabled: true,
			mix: Value::Fixed(1.0),
		}
	}

	pub fn enabled(self, enabled: bool) -> Self {
		Self { enabled, ..self }
	}

	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

#[allow(unused_variables)]
/// Receives input audio from a mixer track and outputs modified audio.
pub trait Effect: Send {
	/// Performs any required setup for the effect.
	///
	/// This is called once when the effect is first added to a track.
	fn init(&mut self, sample_rate: u32) {}

	/// Transforms an input frame.
	/// - `dt` is the time that's elapsed since the previous frame (in seconds)
	/// - `input` is the input audio
	fn process(&mut self, dt: f64, input: Frame) -> Frame;
}
