#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundMetadata {
	pub loop_start: Option<f64>,
	pub semantic_duration: Option<f64>,
}

impl SoundMetadata {
	pub fn new() -> Self {
		Self {
			loop_start: None,
			semantic_duration: None,
		}
	}

	pub fn loop_start(self, loop_start: impl Into<Option<f64>>) -> Self {
		Self {
			loop_start: loop_start.into(),
			..self
		}
	}

	pub fn semantic_duration(self, semantic_duration: impl Into<Option<f64>>) -> Self {
		Self {
			semantic_duration: semantic_duration.into(),
			..self
		}
	}
}

impl Default for SoundMetadata {
	fn default() -> Self {
		Self::new()
	}
}
