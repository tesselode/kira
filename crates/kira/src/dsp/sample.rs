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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct I24(pub i32);

impl Sample for I24 {
	fn into_f32(self) -> f32 {
		self.0 as f32 / ((1 << 24) as f32 / 2.0)
	}
}
