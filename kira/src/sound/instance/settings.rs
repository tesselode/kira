use std::sync::Arc;

use crate::sound::data::SoundData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceLoopStart {
	DefaultForSoundData,
	Custom(f64),
	None,
}

impl InstanceLoopStart {
	pub(crate) fn as_option(self, data: &Arc<dyn SoundData>) -> Option<f64> {
		match self {
			Self::DefaultForSoundData => data.metadata().loop_start,
			Self::Custom(loop_start) => Some(loop_start),
			Self::None => None,
		}
	}
}

impl<T: Into<Option<f64>>> From<T> for InstanceLoopStart {
	fn from(loop_start: T) -> Self {
		match loop_start.into() {
			Some(loop_start) => Self::Custom(loop_start),
			None => Self::None,
		}
	}
}

impl Default for InstanceLoopStart {
	fn default() -> Self {
		Self::DefaultForSoundData
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceSettings {
	pub start_position: f64,
	pub playback_rate: f64,
	pub reverse: bool,
	pub loop_start: InstanceLoopStart,
}

impl InstanceSettings {
	pub fn new() -> Self {
		Self {
			start_position: 0.0,
			playback_rate: 1.0,
			reverse: false,
			loop_start: InstanceLoopStart::default(),
		}
	}

	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	pub fn playback_rate(self, playback_rate: f64) -> Self {
		Self {
			playback_rate,
			..self
		}
	}

	pub fn reverse(self) -> Self {
		Self {
			reverse: true,
			..self
		}
	}

	pub fn loop_start(self, loop_start: impl Into<InstanceLoopStart>) -> Self {
		Self {
			loop_start: loop_start.into(),
			..self
		}
	}
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self::new()
	}
}
