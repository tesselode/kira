use std::convert::TryInto;

use super::{EndPosition, Region};

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
	pub reverse: bool,
	pub playing: bool,
}

impl Transport {
	pub fn new(
		playback_region: Region,
		loop_region: Option<Region>,
		reverse: bool,
		sample_rate: u32,
		num_frames: usize,
	) -> Self {
		let playback_start = playback_region.start.into_samples(sample_rate);
		let playback_end = match playback_region.end {
			EndPosition::EndOfAudio => (num_frames - 1)
				.try_into()
				.expect("could not convert usize to i64"),
			EndPosition::Custom(end_position) => end_position.into_samples(sample_rate),
		};
		let playback_region = (playback_start, playback_end);
		let loop_region = loop_region.map(|loop_region| {
			let loop_start = loop_region.start.into_samples(sample_rate);
			let loop_end = match loop_region.end {
				EndPosition::EndOfAudio => num_frames
					.try_into()
					.expect("could not convert usize to i64"),
				EndPosition::Custom(end_position) => end_position.into_samples(sample_rate),
			};
			(loop_start, loop_end)
		});
		Self {
			position: if reverse {
				playback_region.1
			} else {
				playback_region.0
			},
			playback_region,
			loop_region,
			reverse,
			playing: true,
		}
	}

	pub fn set_playback_region(
		&mut self,
		playback_region: Region,
		sample_rate: u32,
		num_frames: usize,
	) {
		let playback_start = playback_region.start.into_samples(sample_rate);
		let playback_end = match playback_region.end {
			EndPosition::EndOfAudio => (num_frames - 1)
				.try_into()
				.expect("could not convert usize to i64"),
			EndPosition::Custom(end_position) => end_position.into_samples(sample_rate),
		};
		self.playback_region = (playback_start, playback_end);
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
				EndPosition::EndOfAudio => num_frames
					.try_into()
					.expect("could not convert usize to i64"),
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
		if self.position < self.playback_region.0 || self.position > self.playback_region.1 {
			self.playing = false;
		}
	}
}
