pub trait Sample: Copy {
	fn into_f32(self) -> f32;
}
