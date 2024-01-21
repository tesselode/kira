use super::{EndPosition, Region};

#[cfg(test)]
mod test;

pub struct Transport {
	pub num_frames: i64,
	pub position: i64,
	/// The start and end frames of the sound that should be looped. The upper bound
	/// is *exclusive*.
	pub loop_region: Option<(i64, i64)>,
	pub playing: bool,
}

impl Transport {
	pub fn new(
		num_frames: i64,
		loop_region: Option<Region>,
		reverse: bool,
		sample_rate: u32,
	) -> Self {
		let loop_region = loop_region.map(|loop_region| {
			let loop_start = loop_region.start.into_samples(sample_rate);
			let loop_end = match loop_region.end {
				EndPosition::EndOfAudio => num_frames,
				EndPosition::Custom(end_position) => end_position.into_samples(sample_rate),
			};
			(loop_start, loop_end)
		});
		Self {
			num_frames,
			position: if reverse { num_frames - 1 } else { 0 },
			loop_region,
			playing: true,
		}
	}

	pub fn set_loop_region(
		&mut self,
		loop_region: Option<Region>,
		sample_rate: u32,
		num_frames: i64,
	) {
		self.loop_region = loop_region.map(|loop_region| {
			let loop_start = loop_region.start.into_samples(sample_rate);
			let loop_end = match loop_region.end {
				EndPosition::EndOfAudio => num_frames,
				EndPosition::Custom(end_position) => end_position.into_samples(sample_rate),
			};
			(loop_start, loop_end)
		});
	}

	pub fn increment_position(&mut self) {
		self.position += 1;
		if let Some((loop_start, loop_end)) = self.loop_region {
			while self.position >= loop_end {
				self.position -= loop_end - loop_start;
			}
		}
		if self.position >= self.num_frames {
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
		if self.position < 0 {
			self.playing = false;
		}
	}

	pub fn seek_to(&mut self, mut position: i64) {
		if let Some((loop_start, loop_end)) = self.loop_region {
			if position > self.position {
				while position >= loop_end {
					position -= loop_end - loop_start;
				}
			} else {
				while position < loop_start {
					position += loop_end - loop_start;
				}
			}
		}
		self.position = position;
		if self.position < 0 || self.position >= self.num_frames {
			self.playing = false;
		}
	}
}
