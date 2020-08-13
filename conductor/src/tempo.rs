#[derive(Debug, Copy, Clone)]
pub struct Tempo(pub f32);

impl Tempo {
	pub fn beats_to_seconds(&self, beats: f32) -> f32 {
		(self.0 / 60.0) * beats
	}
}
