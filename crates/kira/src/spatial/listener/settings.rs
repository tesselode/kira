use crate::{
	math::{Quaternion, Vec3},
	track::TrackId,
};

pub struct ListenerSettings {
	pub position: Vec3,
	pub orientation: Quaternion,
	pub track: TrackId,
}

impl ListenerSettings {
	pub fn new() -> Self {
		Self {
			position: Vec3::default(),
			orientation: Quaternion::default(),
			track: TrackId::Main,
		}
	}

	pub fn position(self, position: Vec3) -> Self {
		Self { position, ..self }
	}

	pub fn orientation(self, orientation: Quaternion) -> Self {
		Self {
			orientation,
			..self
		}
	}

	pub fn track(self, track: impl Into<TrackId>) -> Self {
		Self {
			track: track.into(),
			..self
		}
	}
}

impl Default for ListenerSettings {
	fn default() -> Self {
		Self::new()
	}
}
