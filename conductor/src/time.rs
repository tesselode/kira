/// Represents a duration of time.
#[derive(Copy, Clone, Debug)]
pub enum Time {
	/// Represents a duration of time in seconds.
	Seconds(f32),
	/// Represents a duration of time in beats.
	Beats(f32),
}

impl Time {
	/// Gets the time in seconds.
	pub fn in_seconds(&self, tempo: f32) -> f32 {
		match self {
			Time::Seconds(seconds) => *seconds,
			Time::Beats(beats) => beats * (60.0 / tempo),
		}
	}
}
