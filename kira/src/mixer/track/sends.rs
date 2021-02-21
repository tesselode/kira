use indexmap::IndexMap;

use crate::{parameter::Parameters, CachedValue, Value};

use super::SendTrackId;

#[derive(Debug, Clone)]
pub struct TrackSends {
	sends: IndexMap<SendTrackId, CachedValue<f64>>,
}

impl TrackSends {
	pub fn new() -> Self {
		Self {
			sends: IndexMap::new(),
		}
	}

	pub fn iter(&self) -> indexmap::map::Iter<SendTrackId, CachedValue<f64>> {
		self.sends.iter()
	}

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
