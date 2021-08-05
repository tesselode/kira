/// Settings for [`StaticSound`](super::StaticSound).
pub struct StaticSoundSettings {
	pub default_loop_start: Option<f64>,
}

impl StaticSoundSettings {
	/// Creates a new [`StaticSoundSettings`] with the default settings.
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

impl Default for StaticSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
