//! Traits for sources of audio.

pub mod static_sound;

use crate::{clock::Clocks, dsp::Frame, parameter::Parameters, track::TrackId};

/// Represents a source of audio that is loaded, but not yet playing.
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

/// Represents an actively playing sound.
pub trait Sound: Send {
	/// Returns the mixer track that this sound's audio should be routd to.
	fn track(&mut self) -> TrackId;

	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Produces the next [`Frame`] of audio.
	fn process(&mut self, dt: f64, parameters: &Parameters, clocks: &Clocks) -> Frame;

	/// Returns `true` if the sound is finished and can be unloaded.
	fn finished(&self) -> bool;
}
