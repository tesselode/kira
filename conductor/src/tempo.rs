#[derive(Debug, Copy, Clone)]
pub struct Tempo(pub f64);

impl Tempo {
	pub fn beats_to_seconds(&self, beats: f64) -> f64 {
		(60.0 / self.0) * beats
	}
}
