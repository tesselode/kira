use super::renderer::Renderer;

pub trait Backend {
	type InitError;

	fn sample_rate(&mut self) -> u32;

	fn init(&mut self, renderer: Renderer) -> Result<(), Self::InitError>;
}
