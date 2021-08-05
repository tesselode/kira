mod mock;

pub use mock::MockBackend;

use super::{resources::UnusedResourceCollector, Renderer};

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
