/*!
Sources of audio.
*/

#[cfg(feature = "symphonia")]
mod error;
pub mod static_sound;
#[cfg(all(feature = "symphonia", not(target_arch = "wasm32")))]
pub mod streaming;

#[cfg(feature = "symphonia")]
pub use error::*;

use crate::{clock::clock_info::ClockInfoProvider, dsp::Frame, OutputDestination};

/// A source of audio that is loaded, but not yet playing.
pub trait SoundData {
	/// Errors that can occur when starting the sound.
	type Error;

	/// The type that can be used to control the sound once
	/// it has started.
	type Handle;

	/// Converts the loaded sound into a live, playing sound
	/// and a handle to control it.
	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error>;
}

/// An actively playing sound.
#[allow(unused_variables)]
pub trait Sound: Send {
	/// Returns the destination that this sound's audio should be routed to.
	fn output_destination(&mut self) -> OutputDestination;

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Produces the next [`Frame`] of audio.
	fn process(&mut self, dt: f64, clock_info_provider: &ClockInfoProvider) -> Frame;

	/// Returns `true` if the sound is finished and can be unloaded.
	fn finished(&self) -> bool;
}
