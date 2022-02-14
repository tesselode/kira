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

	/// Starts the backend and returns itself and the initial sample rate.
	fn setup(settings: Self::Settings) -> Result<(Self, u32), Self::Error>;

	/// Sends the renderer to the backend to start audio playback.
	fn start(&mut self, renderer: Renderer) -> Result<(), Self::Error>;
}
