pub mod mock;

use super::{renderer::Renderer, resources::UnusedResourceCollector};

pub trait Backend {
	type InitError;

	fn sample_rate(&mut self) -> u32;

	fn init(
		&mut self,
		renderer: Renderer,
		unused_resource_collector: UnusedResourceCollector,
	) -> Result<(), Self::InitError>;
}
