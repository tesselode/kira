use crate::Tempo;

/// Settings for the metronome.
#[derive(Debug, Clone)]
pub struct MetronomeSettings {
	/// The tempo of the metronome (in beats per minute).
	pub tempo: Tempo,
	/// Which intervals (in beats) the metronome should emit events for.
	///
	/// For example, if this is set to `vec![0.25, 0.5, 1.0]`, then
	/// the audio manager will receive `MetronomeIntervalPassed` events
	/// every quarter of a beat, half of a beat, and beat.
	pub interval_events_to_emit: Vec<f64>,
}

impl MetronomeSettings {
	/// Creates a new `MetronomeSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the tempo of the metronome.
	pub fn tempo(self, tempo: impl Into<Tempo>) -> Self {
		Self {
			tempo: tempo.into(),
			..self
		}
	}

	/// Sets which intervals (in beats) the metronome should emit events for.
	pub fn interval_events_to_emit(self, interval_events_to_emit: impl Into<Vec<f64>>) -> Self {
		Self {
			interval_events_to_emit: interval_events_to_emit.into(),
			..self
		}
	}
}

impl Default for MetronomeSettings {
	fn default() -> Self {
		Self {
			tempo: Tempo(120.0).into(),
			interval_events_to_emit: vec![],
		}
	}
}
