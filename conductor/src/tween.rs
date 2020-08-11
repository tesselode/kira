use crate::time::Time;

#[derive(Debug, Copy, Clone)]
pub struct Tween<T>(pub T);

impl Tween<Time> {
	pub fn in_seconds(&self, tempo: f32) -> Tween<f32> {
		Tween(self.0.in_seconds(tempo))
	}
}
