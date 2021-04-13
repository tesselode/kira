use crate::mixer::track::{handle::TrackHandle, TrackInput};

#[derive(Clone)]
pub struct InstanceSettings {
	pub(crate) track: Option<TrackInput>,
}

impl InstanceSettings {
	pub fn new() -> Self {
		Self { track: None }
	}

	pub fn track(self, track: &TrackHandle) -> Self {
		Self {
			track: Some(track.input()),
			..self
		}
	}
}
