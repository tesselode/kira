mod settings;

pub use settings::*;

use crate::{
	backend::{Backend, DefaultBackend},
	renderer::Renderer,
	resources::{create_resources, ResourceControllers},
	sound::SoundData,
	PlaySoundError,
};

/// Controls audio from gameplay code.
pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
	resource_controllers: ResourceControllers,
}

impl<B: Backend> AudioManager<B> {
	/**
	Creates a new [`AudioManager`].

	# Examples

	Create an [`AudioManager`] using the [`DefaultBackend`] and the
	default settings:

	```no_run
	use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};

	let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Create an [`AudioManager`] with a reverb effect on the main mixer track:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		track::MainTrackBuilder,
		effect::reverb::ReverbBuilder,
	};

	let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings {
		main_track_builder: MainTrackBuilder::new().with_effect(ReverbBuilder::new()),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
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

	/// Plays a sound.
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
}
