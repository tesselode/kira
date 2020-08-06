#[derive(Copy, Clone, Debug)]
pub enum Time {
	Seconds(f32),
	Beats(f32),
}

impl Time {
	pub fn in_seconds(&self, tempo: f32) -> f32 {
		match self {
			Time::Seconds(seconds) => *seconds,
			Time::Beats(beats) => beats * (60.0 / tempo),
		}
	}
}
