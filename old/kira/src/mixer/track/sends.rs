use indexmap::IndexMap;

use crate::{CachedValue, Value};

use super::SendTrackId;

/// A mapping of send tracks to volume levels.
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct TrackSends {
	sends: IndexMap<SendTrackId, Value<f64>>,
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
	pub fn iter(&self) -> indexmap::map::Iter<SendTrackId, Value<f64>> {
		self.sends.iter()
	}

	/// Adds a `SendTrackId` to the map with the volume level of the signal
	/// to send to that track.
	pub fn add(
		mut self,
		send_track: impl Into<SendTrackId>,
		volume: impl Into<Value<f64>>,
	) -> Self {
		self.sends.insert(send_track.into(), volume.into());
		self
	}

	pub(crate) fn to_map(&self) -> IndexMap<SendTrackId, CachedValue<f64>> {
		let mut map = IndexMap::new();
		for (id, volume) in self.iter() {
			map.insert(*id, CachedValue::new(*volume, 1.0));
		}
		map
	}
}

impl Default for TrackSends {
	fn default() -> Self {
		Self::new()
	}
}
