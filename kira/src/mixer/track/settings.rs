use crate::value::Value;

use super::{handle::TrackHandle, TrackInput};

#[derive(Clone)]
pub struct SubTrackSettings {
	pub(crate) volume: Value<f64>,
	pub(crate) parent: Option<TrackInput>,
	pub(crate) num_effects: usize,
}

impl SubTrackSettings {
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			parent: None,
			num_effects: 10,
		}
	}

	pub fn volume(self, volume: impl Into<Value<f64>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	pub fn parent(self, parent: &TrackHandle) -> Self {
		Self {
			parent: Some(parent.input()),
			..self
		}
	}

	pub fn num_effects(self, num_effects: usize) -> Self {
		Self {
			num_effects,
			..self
		}
	}
}
