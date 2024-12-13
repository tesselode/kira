mod settings;

pub use settings::*;

use crate::{
	backend::{Backend, DefaultBackend},
	clock::{Clock, ClockHandle, ClockId, ClockSpeed},
	renderer::Renderer,
	resources::{clocks::buffered_clock::BufferedClock, create_resources, ResourceControllers},
	sound::SoundData,
	PlaySoundError, ResourceLimitReached,
};

pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
	resource_controllers: ResourceControllers,
}

impl<B: Backend> AudioManager<B> {
	pub fn new(settings: AudioManagerSettings<B>) -> Result<Self, B::Error> {
		let (mut backend, sample_rate) = B::setup(settings.backend_settings)?;
		let (resources, resource_controllers) = create_resources(sample_rate, settings.capacities);
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

	pub fn add_clock(&mut self, speed: ClockSpeed) -> Result<ClockHandle, ResourceLimitReached> {
		let key = self.resource_controllers.clock_controller.try_reserve()?;
		let id = ClockId(key);
		let (clock, handle) = Clock::new(speed, id);
		self.resource_controllers
			.clock_controller
			.insert_with_key(key, BufferedClock::new(clock));
		Ok(handle)
	}

	/// Returns a mutable reference to this manager's backend.
	#[must_use]
	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}
}
