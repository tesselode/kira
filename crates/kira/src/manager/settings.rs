use crate::backend::Backend;

/// Settings for an [`AudioManager`](super::AudioManager).
pub struct AudioManagerSettings<B: Backend> {
	/* /// Specifies how many of each resource type an audio context
	/// can have.
	pub capacities: Capacities, */
	/* /// Configures the main mixer track.
	pub main_track_builder: MainTrackBuilder, */
	/// Configures the backend.
	pub backend_settings: B::Settings,
}

impl<B: Backend> Default for AudioManagerSettings<B>
where
	B::Settings: Default,
{
	fn default() -> Self {
		Self {
			// capacities: Capacities::default(),
			// main_track_builder: MainTrackBuilder::default(),
			backend_settings: B::Settings::default(),
		}
	}
}
