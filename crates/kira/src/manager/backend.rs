//! Communication between Kira and a low-level audio API.

mod mock;
mod renderer;
pub(crate) mod resources;

pub use mock::*;
pub use renderer::*;

/// Connects a [`Renderer`] to a lower level audio API.
pub trait Backend: Sized {
	/// Settings for this backend.
	type Settings;

	/// Errors that can occur when using this backend.
	type Error;

	/// Starts the backend.
	fn setup(settings: Self::Settings) -> Result<Self, Self::Error>;

	/// Sends the renderer to the backend to start audio playback.
	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error>;

	/// Returns the sample rate that the backend is running at.
	fn sample_rate(&self) -> u32;
}
