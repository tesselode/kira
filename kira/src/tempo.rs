use atomig::Atom;

/// A tempo, or speed, of some music (in beats per minute).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tempo(pub f64);

impl Atom for Tempo {
	type Repr = u64;

	fn pack(self) -> Self::Repr {
		self.0.to_bits()
	}

	fn unpack(src: Self::Repr) -> Self {
		Tempo(f64::from_bits(src))
	}
}

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
