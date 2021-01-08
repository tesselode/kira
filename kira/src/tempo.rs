/// Represents a tempo, or speed, of some music (in beats per minute).
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
pub struct Tempo(pub f64);

impl Tempo {
	/// Converts a number of beats at this tempo to a length
	/// of time in seconds.
	pub fn beats_to_seconds(&self, beats: f64) -> f64 {
		(60.0 / self.0) * beats
	}
}

impl From<f64> for Tempo {
	fn from(bpm: f64) -> Self {
		Self(bpm)
	}
}

impl Into<f64> for Tempo {
	fn into(self) -> f64 {
		self.0
	}
}
