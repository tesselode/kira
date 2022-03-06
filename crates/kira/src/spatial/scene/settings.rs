pub struct SpatialSceneSettings {
	pub emitter_capacity: usize,
}

impl SpatialSceneSettings {
	pub fn new() -> Self {
		Self {
			emitter_capacity: 128,
		}
	}

	pub fn emitter_capacity(self, emitter_capacity: usize) -> Self {
		Self {
			emitter_capacity,
			..self
		}
	}
}

impl Default for SpatialSceneSettings {
	fn default() -> Self {
		Self::new()
	}
}
