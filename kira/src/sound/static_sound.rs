mod settings;

pub use settings::*;

use crate::{frame::Frame, util};

use std::time::Duration;

use super::Sound;

/// A chunk of audio data loaded into memory all at once.
pub struct StaticSound {
	sample_rate: u32,
	duration: Duration,
	frames: Vec<Frame>,
	default_loop_start: Option<f64>,
}

impl StaticSound {
	/// Creates a new [`StaticSound`] from raw sample data.
	pub fn from_frames(
		sample_rate: u32,
		frames: Vec<Frame>,
		settings: StaticSoundSettings,
	) -> Self {
		let duration = Duration::from_secs_f64(frames.len() as f64 / sample_rate as f64);
		Self {
			sample_rate,
			frames,
			duration,
			default_loop_start: settings.default_loop_start,
		}
	}
}

impl Sound for StaticSound {
	fn duration(&self) -> Duration {
		self.duration
	}

	fn frame_at_position(&self, position: f64) -> Frame {
		let sample_position = self.sample_rate as f64 * position;
		let fraction = (sample_position % 1.0) as f32;
		let current_sample_index = sample_position as usize;
		let previous = if current_sample_index == 0 {
			Frame::from_mono(0.0)
		} else {
			*self
				.frames
				.get(current_sample_index - 1)
				.unwrap_or(&Frame::from_mono(0.0))
		};
		let current = *self
			.frames
			.get(current_sample_index)
			.unwrap_or(&Frame::from_mono(0.0));
		let next_1 = *self
			.frames
			.get(current_sample_index + 1)
			.unwrap_or(&Frame::from_mono(0.0));
		let next_2 = *self
			.frames
			.get(current_sample_index + 2)
			.unwrap_or(&Frame::from_mono(0.0));
		util::interpolate_frame(previous, current, next_1, next_2, fraction)
	}

	fn default_loop_start(&self) -> Option<f64> {
		self.default_loop_start
	}
}
