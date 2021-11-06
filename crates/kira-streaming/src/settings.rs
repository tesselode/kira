use kira::LoopBehavior;

pub struct StreamingSoundSettings {
	pub loop_behavior: Option<LoopBehavior>,
}

impl StreamingSoundSettings {
	pub fn new() -> Self {
		Self {
			loop_behavior: None,
		}
	}

	pub fn loop_behavior(self, loop_behavior: impl Into<Option<LoopBehavior>>) -> Self {
		Self {
			loop_behavior: loop_behavior.into(),
		}
	}
}

impl Default for StreamingSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
