use crate::track::TrackBuilder;

use super::backend::Backend;

/// Specifies how many of each resource type an audio context
/// can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Capacities {
	/// The number of resources (sounds, clocks, etc.) that be sent to the
	/// renderer at a time.
	pub command_capacity: usize,
	/// The maximum number of sounds that can be playing at a time.
	pub sound_capacity: u16,
	/// The maximum number of mixer sub-tracks that can exist at a time.
	pub sub_track_capacity: u16,
	/// The maximum number of clocks that can exist at a time.
	pub clock_capacity: u16,
	/// The maximum number of spatial scenes that can exist at a time.
	pub spatial_scene_capacity: u16,
	/// The maximum number of modulators that can exist at a time.
	pub modulator_capacity: u16,
}

impl Default for Capacities {
	fn default() -> Self {
		Self {
			command_capacity: 128,
			sound_capacity: 128,
			sub_track_capacity: 128,
			clock_capacity: 8,
			spatial_scene_capacity: 8,
			modulator_capacity: 16,
		}
	}
}

/// Settings for an [`AudioManager`](super::AudioManager).
pub struct AudioManagerSettings<B: Backend> {
	/// Specifies how many of each resource type an audio context
	/// can have.
	pub capacities: Capacities,
	/// Configures the main mixer track.
	pub main_track_builder: TrackBuilder,
	/// Configures the backend.
	pub backend_settings: B::Settings,
}

impl<B: Backend> Default for AudioManagerSettings<B>
where
	B::Settings: Default,
{
	fn default() -> Self {
		Self {
			capacities: Capacities::default(),
			main_track_builder: TrackBuilder::default(),
			backend_settings: B::Settings::default(),
		}
	}
}
