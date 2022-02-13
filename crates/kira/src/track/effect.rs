//! Modifies audio signals.

pub mod delay;
pub mod distortion;
pub mod filter;
pub mod reverb;

use crate::{clock::ClockTime, dsp::Frame};

/// Configures an effect.
pub trait EffectBuilder {
	/// Allows the user to control the effect from gameplay code.
	type Handle;

	/// Creates the effect and a handle to the effect.
	fn build(self) -> (Box<dyn Effect>, Self::Handle);
}

/// Receives input audio from a mixer track and outputs modified audio.
#[allow(unused_variables)]
pub trait Effect: Send + Sync {
	/// Called when the effect is first sent to the renderer.
	fn init(&mut self, sample_rate: u32) {}

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Transforms an input [`Frame`].
	/// - `input` is the input audio
	/// - `dt` is the time that's elapsed since the previous round of
	/// processing (in seconds)
	fn process(&mut self, input: Frame, dt: f64) -> Frame;

	/// Called whenever a [clock](crate::clock) ticks.
	fn on_clock_tick(&mut self, time: ClockTime) {}
}
