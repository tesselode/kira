//! Important types for connecting Kira to low-level audio
//! APIs.

mod mock;
mod renderer;
pub(crate) mod resources;

pub use mock::MockBackend;
pub use renderer::*;
pub use resources::UnusedResourceCollector;

/// Connects a [`Renderer`] to a lower level audio API.
pub trait Backend {
	/// An error that can occur when the backend is being initialized.
	type InitError;

	/// Returns the sample rate that the [`Renderer`] should run at.
	fn sample_rate(&mut self) -> u32;

	/// Initializes the [`Backend`].
	fn init(
		&mut self,
		renderer: Renderer,
		unused_resource_collector: UnusedResourceCollector,
	) -> Result<(), Self::InitError>;
}
