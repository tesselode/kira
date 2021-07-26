#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundMetadata {
	pub loop_start: Option<f64>,
	pub semantic_duration: Option<f64>,
}

impl Default for SoundMetadata {
	fn default() -> Self {
		Self {
			loop_start: None,
			semantic_duration: None,
		}
	}
}
