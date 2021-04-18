use crate::{
	mixer::track::{handle::TrackHandle, TrackInput},
	value::Value,
};

#[derive(Clone)]
pub struct InstanceSettings {
	pub(crate) volume: Value<f64>,
	pub(crate) playback_rate: Value<f64>,
	pub(crate) panning: Value<f64>,
	pub(crate) track: Option<TrackInput>,
}

impl InstanceSettings {
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			playback_rate: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			track: None,
		}
	}

	pub fn volume(self, volume: impl Into<Value<f64>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	pub fn playback_rate(self, playback_rate: impl Into<Value<f64>>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	pub fn panning(self, panning: impl Into<Value<f64>>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	pub fn track(self, track: &TrackHandle) -> Self {
		Self {
			track: Some(track.input()),
			..self
		}
	}
}
