use crate::arena::Key;

/// Configures a modulator.
pub trait ModulatorBuilder {
	/// Allows the user to control the modulator from gameplay code.
	type Handle;

	/// Creates the modulator and a handle to the modulator.
	#[must_use]
	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle);
}

/// Produces a stream of values that a parameter can be linked to.
pub trait Modulator: Send {
	/// Called whenever a new batch of audio samples is requested by the backend.
	///
	/// This is a good place to put code that needs to run fairly frequently,
	/// but not for every single audio sample.
	fn on_start_processing(&mut self) {}

	/// Updates the modulator.
	///
	/// `dt` is the time that's elapsed since the previous round of
	/// processing (in seconds).
	fn update(&mut self, dt: f64);

	/// Returns the current output of the modulator.
	#[must_use]
	fn value(&self) -> f64;

	/// Whether the modulator can be removed from the audio context.
	#[must_use]
	fn finished(&self) -> bool;
}

/// A unique identifier for a modulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModulatorId(pub(crate) Key);
