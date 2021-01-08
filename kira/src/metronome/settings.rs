use crate::{Tempo, Value};

use super::MetronomeId;

#[derive(Debug, Clone)]
/// Settings for the metronome.
pub struct MetronomeSettings {
	pub id: MetronomeId,
	/// The tempo of the metronome (in beats per minute).
	pub tempo: Value<Tempo>,
	/// Which intervals (in beats) the metronome should emit events for.
	///
	/// For example, if this is set to `vec![0.25, 0.5, 1.0]`, then
	/// the audio manager will receive `MetronomeIntervalPassed` events
	/// every quarter of a beat, half of a beat, and beat.
	pub interval_events_to_emit: Vec<f64>,
	pub event_queue_capacity: usize,
}

impl MetronomeSettings {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn id(self, id: impl Into<MetronomeId>) -> Self {
		Self {
			id: id.into(),
			..self
		}
	}

	pub fn tempo(self, tempo: impl Into<Value<Tempo>>) -> Self {
		Self {
			tempo: tempo.into(),
			..self
		}
	}

	pub fn interval_events_to_emit(self, interval_events_to_emit: impl Into<Vec<f64>>) -> Self {
		Self {
			interval_events_to_emit: interval_events_to_emit.into(),
			..self
		}
	}

	pub fn event_queue_capacity(self, event_queue_capacity: usize) -> Self {
		Self {
			event_queue_capacity,
			..self
		}
	}
}

impl Default for MetronomeSettings {
	fn default() -> Self {
		Self {
			id: MetronomeId::new(),
			tempo: Tempo(120.0).into(),
			interval_events_to_emit: vec![],
			event_queue_capacity: 10,
		}
	}
}
