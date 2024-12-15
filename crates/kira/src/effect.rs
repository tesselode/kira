/*!
Modifies audio signals.

Any type that implements [`EffectBuilder`] can be added to a mixer track by
using [`TrackBuilder::add_effect`](crate::track::TrackBuilder::add_effect). Kira
comes with a number of commonly used effects.

If needed, you can create custom effects by implementing the [`EffectBuilder`]
and [`Effect`] traits.
*/

// pub mod compressor;
// pub mod delay;
// pub mod distortion;
// pub mod eq_filter;
// pub mod filter;
pub mod panning_control;
// pub mod reverb;
pub mod volume_control;

use crate::{frame::Frame, info::Info};

/// Configures an effect.
pub trait EffectBuilder {
	/// Allows the user to control the effect from gameplay code.
	type Handle;

	/// Creates the effect and a handle to the effect.
	#[must_use]
	fn build(self) -> (Box<dyn Effect>, Self::Handle);
}

/// Receives input audio from a mixer track and outputs modified audio.
///
/// For performance reasons, avoid allocating and deallocating in any methods
/// of this trait besides [`on_change_sample_rate`](Effect::on_change_sample_rate).
#[allow(unused_variables)]
pub trait Effect: Send {
	/// Called when the effect is first sent to the renderer.
	fn init(&mut self, sample_rate: u32) {}

	/// Called when the sample rate of the renderer is changed.
	fn on_change_sample_rate(&mut self, sample_rate: u32) {}

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Transforms a slice of input [`Frame`]s.
	///
	/// `dt` is the time that's elapsed since the previous round of
	/// processing (in seconds).
	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info);
}
