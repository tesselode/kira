use indexmap::IndexMap;

use crate::{parameter::Parameters, CachedValue, Value};

use super::SendTrackId;

/// A mapping of send tracks to volume levels.
#[derive(Debug, Clone)]
pub struct TrackSends {
	sends: IndexMap<SendTrackId, CachedValue<f64>>,
}

impl TrackSends {
	/// Creates a new, empty `TrackSends` struct.
	pub fn new() -> Self {
		Self {
			sends: IndexMap::new(),
		}
	}

	/// Returns an iterator over the pairs of `SendTrackId`s and volume levels
	/// in the map.
	pub fn iter(&self) -> indexmap::map::Iter<SendTrackId, CachedValue<f64>> {
		self.sends.iter()
	}

	/// Adds a `SendTrackId` to the map with the volume level of the signal
	/// to send to that track.
	pub fn add(
		mut self,
		send_track: impl Into<SendTrackId>,
		volume: impl Into<Value<f64>>,
	) -> Self {
		self.sends
			.insert(send_track.into(), CachedValue::new(volume.into(), 1.0));
		self
	}

	pub(crate) fn update(&mut self, parameters: &Parameters) {
		for (_, volume) in &mut self.sends {
			volume.update(parameters);
		}
	}
}

impl Default for TrackSends {
	fn default() -> Self {
		Self::new()
	}
}
