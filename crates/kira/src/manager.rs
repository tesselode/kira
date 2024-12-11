mod settings;

pub use settings::*;

use crate::{
	backend::{Backend, DefaultBackend},
	renderer::Renderer,
	resources::{create_resources, ResourceControllers},
	sound::SoundData,
	PlaySoundError,
};

pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
	resource_controllers: ResourceControllers,
}

impl<B: Backend> AudioManager<B> {
	pub fn new(settings: AudioManagerSettings<B>) -> Result<Self, B::Error> {
		let (mut backend, sample_rate) = B::setup(settings.backend_settings)?;
		let (resources, resource_controllers) = create_resources(sample_rate);
		let renderer = Renderer::new(sample_rate, resources);
		backend.start(renderer)?;
		Ok(Self {
			backend,
			resource_controllers,
		})
	}

	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		let (sound, handle) = sound_data
			.into_sound()
			.map_err(PlaySoundError::IntoSoundError)?;
		self.resource_controllers
			.sound_controller
			.insert(sound)
			.map_err(|_| PlaySoundError::SoundLimitReached)?;
		Ok(handle)
	}

	/// Returns a mutable reference to this manager's backend.
	#[must_use]
	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}
}
