#[cfg(test)]
mod test;

/// A point in time in a piece of audio.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaybackPosition {
	/// The time in seconds.
	Seconds(f64),
	/// The time in samples (individual audio data points).
	Samples(usize),
}

impl PlaybackPosition {
	#[must_use]
	pub(crate) fn into_samples(self, sample_rate: u32) -> usize {
		match self {
			PlaybackPosition::Seconds(seconds) => (seconds * sample_rate as f64).round() as usize,
			PlaybackPosition::Samples(samples) => samples,
		}
	}
}

impl From<f64> for PlaybackPosition {
	fn from(v: f64) -> Self {
		Self::Seconds(v)
	}
}
