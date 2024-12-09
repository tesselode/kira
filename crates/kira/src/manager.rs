mod settings;

pub use settings::*;

use crate::{
	backend::{Backend, DefaultBackend},
	renderer::Renderer,
};

/// Controls audio from gameplay code.
pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
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
		let renderer = Renderer::new(sample_rate);
		backend.start(renderer)?;
		Ok(Self { backend })
	}
}
