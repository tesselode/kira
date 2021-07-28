use std::{sync::Arc, time::Duration};

use crate::{sound::data::SoundData, value::Value};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceLoopStart {
	FromMetadata,
	Custom(f64),
	None,
}

impl InstanceLoopStart {
	pub(crate) fn as_option(self, data: &Arc<dyn SoundData>) -> Option<f64> {
		match self {
			Self::FromMetadata => data.metadata().loop_start,
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
		Self::FromMetadata
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceSettings {
	pub delay: Duration,
	pub start_position: f64,
	pub volume: Value,
	pub playback_rate: Value,
	pub panning: Value,
	pub reverse: bool,
	pub loop_start: InstanceLoopStart,
}

impl InstanceSettings {
	pub fn new() -> Self {
		Self {
			delay: Duration::default(),
			start_position: 0.0,
			volume: Value::Fixed(1.0),
			playback_rate: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			reverse: false,
			loop_start: InstanceLoopStart::default(),
		}
	}

	pub fn delay(self, delay: Duration) -> Self {
		Self { delay, ..self }
	}

	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	pub fn volume(self, volume: impl Into<Value>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	pub fn playback_rate(self, playback_rate: impl Into<Value>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	pub fn panning(self, panning: impl Into<Value>) -> Self {
		Self {
			panning: panning.into(),
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
