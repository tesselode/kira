use crate::tempo::Tempo;

/// Represents a duration of time.
#[derive(Copy, Clone, Debug)]
pub enum Duration {
	/// Represents a duration of time in seconds.
	Seconds(f64),
	/// Represents a duration of time in beats.
	Beats(f64),
}

impl Duration {
	/// Gets the time in seconds.
	pub fn in_seconds(&self, tempo: Tempo) -> f64 {
		match self {
			Duration::Seconds(seconds) => *seconds,
			Duration::Beats(beats) => tempo.beats_to_seconds(*beats),
		}
	}
}
