use kira::LoopBehavior;

pub struct StreamingSoundSettings {
	pub start_position: f64,
	pub loop_behavior: Option<LoopBehavior>,
}

impl StreamingSoundSettings {
	pub fn new() -> Self {
		Self {
			start_position: 0.0,
			loop_behavior: None,
		}
	}

	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	pub fn loop_behavior(self, loop_behavior: impl Into<Option<LoopBehavior>>) -> Self {
		Self {
			loop_behavior: loop_behavior.into(),
			..self
		}
	}
}

impl Default for StreamingSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
