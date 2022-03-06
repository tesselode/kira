use crate::track::TrackBuilder;

use super::backend::Backend;

/// Specifies how many of each resource type an audio context
/// can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Capacities {
	/// The number of commands that be sent to the renderer at a time.
	///
	/// Each action you take, like playing a sound or pausing a clock,
	/// queues up one command.
	pub command_capacity: usize,
	/// The maximum number of sounds that can be playing at a time.
	pub sound_capacity: usize,
	/// The maximum number of mixer sub-tracks that can exist at a time.
	pub sub_track_capacity: usize,
	/// The maximum number of clocks that can exist at a time.
	pub clock_capacity: usize,
	/// The maximum number of spatial scenes that can exist at a time.
	pub spatial_scene_capacity: usize,
}

impl Default for Capacities {
	fn default() -> Self {
		Self {
			command_capacity: 128,
			sound_capacity: 128,
			sub_track_capacity: 128,
			clock_capacity: 8,
			spatial_scene_capacity: 8,
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
