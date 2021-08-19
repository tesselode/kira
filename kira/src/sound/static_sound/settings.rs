use crate::loop_behavior::LoopBehavior;

/// Settings for [`StaticSound`](super::StaticSound).
pub struct StaticSoundSettings {
	/// The default loop behavior for the sound, if any.
	pub default_loop_behavior: Option<LoopBehavior>,
}

impl StaticSoundSettings {
	/// Creates a new [`StaticSoundSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			default_loop_behavior: None,
		}
	}

	/// Sets the default loop behavior for the sound, if any.
	pub fn default_loop_behavior(self, loop_behavior: impl Into<Option<LoopBehavior>>) -> Self {
		Self {
			default_loop_behavior: loop_behavior.into(),
		}
	}
}

impl Default for StaticSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
