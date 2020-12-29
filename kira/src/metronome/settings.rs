use crate::{Tempo, Value};

#[derive(Debug, Clone)]
/// Settings for the metronome.
pub struct MetronomeSettings {
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

impl Default for MetronomeSettings {
	fn default() -> Self {
		Self {
			tempo: Tempo(120.0).into(),
			interval_events_to_emit: vec![],
			event_queue_capacity: 10,
		}
	}
}
