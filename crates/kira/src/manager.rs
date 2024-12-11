mod settings;

pub use settings::*;

use crate::{
	backend::{Backend, DefaultBackend},
	renderer::Renderer,
};

pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
}

impl<B: Backend> AudioManager<B> {
	pub fn new(settings: AudioManagerSettings<B>) -> Result<Self, B::Error> {
		let (mut backend, sample_rate) = B::setup(settings.backend_settings)?;
		let renderer = Renderer::new(sample_rate);
		backend.start(renderer)?;
		Ok(Self { backend })
	}
}
