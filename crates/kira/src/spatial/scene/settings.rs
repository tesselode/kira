/// Settings for a spatial scene.
pub struct SpatialSceneSettings {
	/// The maximum number of emitters that can be in the scene at once.
	pub emitter_capacity: u16,
	/// The maximum number of listeners that can be in the scene at once.
	pub listener_capacity: u16,
}

impl SpatialSceneSettings {
	/// Creates a new [`SpatialSceneSettings`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			emitter_capacity: 128,
			listener_capacity: 8,
		}
	}

	/// Sets the maximum number of emitters that can be in the scene at once.
	#[must_use = "This method consumes self and returns a modified SpatialSceneSettings, so the return value should be used"]
	pub fn emitter_capacity(self, emitter_capacity: u16) -> Self {
		Self {
			emitter_capacity,
			..self
		}
	}

	/// Sets the maximum number of listeners that can be in the scene at once.
	#[must_use = "This method consumes self and returns a modified SpatialSceneSettings, so the return value should be used"]
	pub fn listener_capacity(self, listener_capacity: u16) -> Self {
		Self {
			listener_capacity,
			..self
		}
	}
}

impl Default for SpatialSceneSettings {
	fn default() -> Self {
		Self::new()
	}
}
