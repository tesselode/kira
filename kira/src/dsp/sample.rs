pub trait Sample: Copy {
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
