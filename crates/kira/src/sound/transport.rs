// use super::{EndPosition, Region};

use super::{EndPosition, Region};

#[cfg(test)]
mod test;

pub struct Transport {
	pub position: usize,
	/// The start and end frames of the sound that should be looped. The upper bound
	/// is *exclusive*.
	pub loop_region: Option<(usize, usize)>,
	pub playing: bool,
}

impl Transport {
	#[must_use]
	pub fn new(
		start_position: usize,
		loop_region: Option<Region>,
		reverse: bool,
		sample_rate: u32,
		num_frames: usize,
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
			position: if reverse {
				num_frames - 1 - start_position
			} else {
				start_position
			},
			loop_region,
			playing: true,
		}
	}

	pub fn set_loop_region(
		&mut self,
		loop_region: Option<Region>,
		sample_rate: u32,
		num_frames: usize,
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

	pub fn increment_position(&mut self, num_frames: usize) {
		if !self.playing {
			return;
		}
		self.position += 1;
		if let Some((loop_start, loop_end)) = self.loop_region {
			while self.position >= loop_end {
				self.position -= loop_end - loop_start;
			}
		}
		if self.position >= num_frames {
			self.playing = false;
		}
	}

	pub fn decrement_position(&mut self) {
		if !self.playing {
			return;
		}
		if let Some((loop_start, loop_end)) = self.loop_region {
			while self.position <= loop_start {
				self.position += loop_end - loop_start;
			}
		}
		if self.position == 0 {
			self.playing = false;
		} else {
			self.position -= 1;
		}
	}

	pub fn seek_to(&mut self, mut position: usize, num_frames: usize) {
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
		if self.position >= num_frames {
			self.playing = false;
		}
	}
}
