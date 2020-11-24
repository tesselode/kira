use std::ops::Range;

use indexmap::IndexMap;

use crate::{
	sound::{Sound, SoundId},
	util::inverse_lerp,
	util::lerp,
	Frame,
};

#[derive(Debug, Clone)]
pub struct SoundClip {
	pub sound_id: SoundId,
	pub clip_time_range: Range<f64>,
	pub sound_time_range: Range<f64>,
}

impl SoundClip {
	pub fn new(sound_id: SoundId, clip_start_time: f64) -> Self {
		Self {
			sound_id,
			clip_time_range: clip_start_time..(clip_start_time + sound_id.duration()),
			sound_time_range: 0.0..sound_id.duration(),
		}
	}

	pub fn duration(&self) -> f64 {
		self.clip_time_range.end - self.clip_time_range.start
	}

	pub fn stretch(mut self, factor: f64) -> Self {
		self.clip_time_range.end =
			lerp(self.clip_time_range.start, self.clip_time_range.end, factor);
		self
	}

	pub fn trim(mut self, duration: f64) -> Self {
		let new_duration_factor = duration / self.duration();
		self.clip_time_range.end = self.clip_time_range.start + duration;
		self.sound_time_range.end = lerp(
			self.sound_time_range.start,
			self.sound_time_range.end,
			new_duration_factor,
		);
		self
	}

	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Sound>,
	) -> Frame {
		if let Some(sound) = sounds.get(&self.sound_id) {
			let relative_time = inverse_lerp(
				self.clip_time_range.start,
				self.clip_time_range.end,
				position,
			);
			if relative_time < 0.0 || relative_time > 1.0 {
				Frame::from_mono(0.0)
			} else {
				sound.get_frame_at_position(lerp(
					self.sound_time_range.start,
					self.sound_time_range.end,
					relative_time,
				))
			}
		} else {
			Frame::from_mono(0.0)
		}
	}
}
