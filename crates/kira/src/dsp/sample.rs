/// A trait for types that can be used as audio samples.
pub trait Sample: Copy {
	/// Converts the sample into an [`f32`] in the range of
	/// `-1.0..=1.0`.
	fn into_f32(self) -> f32;
}

impl Sample for f32 {
	fn into_f32(self) -> f32 {
		self
	}
}

impl Sample for i16 {
	fn into_f32(self) -> f32 {
		self as f32 / i16::MAX as f32
	}
}

/// A signed 24-bit audio sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct I24(pub i32);

impl Sample for I24 {
	fn into_f32(self) -> f32 {
		self.0 as f32 / ((1 << 24) as f32 / 2.0)
	}
}
