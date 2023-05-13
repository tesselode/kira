/// Settings for a spatial scene.
pub struct SpatialSceneSettings {
	/// The maximum number of emitters that can be in the scene at once.
	pub emitter_capacity: usize,
	/// The maximum number of listeners that can be in the scene at once.
	pub listener_capacity: usize,
}

impl SpatialSceneSettings {
	/// Creates a new [`SpatialSceneSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			emitter_capacity: 128,
			listener_capacity: 8,
		}
	}

	/// Sets the maximum number of emitters that can be in the scene at once.
	pub fn emitter_capacity(self, emitter_capacity: usize) -> Self {
		Self {
			emitter_capacity,
			..self
		}
	}

	/// Sets the maximum number of listeners that can be in the scene at once.
	pub fn listener_capacity(self, listener_capacity: usize) -> Self {
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
