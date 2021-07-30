pub struct StaticSoundDataSettings {
	pub default_loop_start: Option<f64>,
}

impl StaticSoundDataSettings {
	pub fn new() -> Self {
		Self {
			default_loop_start: None,
		}
	}

	pub fn default_loop_start(self, loop_start: impl Into<Option<f64>>) -> Self {
		Self {
			default_loop_start: loop_start.into(),
			..self
		}
	}
}

impl Default for StaticSoundDataSettings {
	fn default() -> Self {
		Self::new()
	}
}
