use crate::track::TrackBuilder;

/// Settings for an [`AudioManager`].
#[non_exhaustive]
pub struct AudioManagerSettings {
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
	/// Configures the main mixer track.
	pub main_track_builder: TrackBuilder,
}

impl AudioManagerSettings {
	/// Creates a new [`AudioManagerSettings`] with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the number of commands that be sent to the renderer at a time.
	///
	/// Each action you take, like playing a sound or pausing a clock,
	/// queues up one command.
	pub fn command_capacity(self, command_capacity: usize) -> Self {
		Self {
			command_capacity,
			..self
		}
	}

	/// Sets the maximum number of sounds that can be playing at a time.
	pub fn sound_capacity(self, sound_capacity: usize) -> Self {
		Self {
			sound_capacity,
			..self
		}
	}

	/// Sets the maximum number of mixer sub-tracks that can exist at a time.
	pub fn sub_track_capacity(self, sub_track_capacity: usize) -> Self {
		Self {
			sub_track_capacity,
			..self
		}
	}

	/// Sets the maximum number of clocks that can exist at a time.
	pub fn clock_capacity(self, clock_capacity: usize) -> Self {
		Self {
			clock_capacity,
			..self
		}
	}

	/// Configures the main mixer track.
	pub fn main_track_builder(self, builder: TrackBuilder) -> Self {
		Self {
			main_track_builder: builder,
			..self
		}
	}
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			command_capacity: 128,
			sound_capacity: 128,
			sub_track_capacity: 128,
			clock_capacity: 8,
			main_track_builder: TrackBuilder::default(),
		}
	}
}
