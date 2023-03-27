#[cfg(test)]
mod test;

pub struct Transport {
	pub position: i64,
	/// The start and end frames of the sound that should be played. The upper bound
	/// is *inclusive*.
	pub playback_region: (i64, i64),
	/// The start and end frames of the sound that should be looped. The upper bound
	/// is *exclusive*.
	pub loop_region: Option<(i64, i64)>,
	pub playing: bool,
}

impl Transport {
	pub fn new(playback_region: (i64, i64), loop_region: Option<(i64, i64)>) -> Self {
		Self {
			position: playback_region.0,
			playback_region,
			loop_region,
			playing: true,
		}
	}

	pub fn increment_position(&mut self) {
		self.position += 1;
		if let Some((loop_start, loop_end)) = self.loop_region {
			while self.position >= loop_end {
				self.position -= loop_end - loop_start;
			}
		}
		if self.position > self.playback_region.1 {
			self.playing = false;
		}
	}

	pub fn decrement_position(&mut self) {
		self.position -= 1;
		if let Some((loop_start, loop_end)) = self.loop_region {
			while self.position < loop_start {
				self.position += loop_end - loop_start;
			}
		}
		if self.position < self.playback_region.0 {
			self.playing = false;
		}
	}
}
