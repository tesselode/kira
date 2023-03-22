use crate::{parameter::Value, PlaybackRate};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundSettings {
	pub playback_rate: Value<PlaybackRate>,
}

impl SoundSettings {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn playback_rate(self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}
}

impl Default for SoundSettings {
	fn default() -> Self {
		Self {
			playback_rate: Value::Fixed(PlaybackRate::Factor(1.0)),
		}
	}
}
