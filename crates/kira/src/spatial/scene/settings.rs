pub struct SpatialSceneSettings {
	pub emitter_capacity: usize,
	pub listener_capacity: usize,
}

impl SpatialSceneSettings {
	pub fn new() -> Self {
		Self {
			emitter_capacity: 128,
			listener_capacity: 8,
		}
	}

	pub fn emitter_capacity(self, emitter_capacity: usize) -> Self {
		Self {
			emitter_capacity,
			..self
		}
	}

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
