use std::ops::Range;

use indexmap::IndexMap;

use crate::{
	sound::{Sound, SoundId},
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

	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Sound>,
	) -> Frame {
		if let Some(sound) = sounds.get(&self.sound_id) {
			let relative_time = (position - self.clip_time_range.start)
				/ (self.clip_time_range.end - self.clip_time_range.start);
			if relative_time < 0.0 || relative_time > 1.0 {
				Frame::from_mono(0.0)
			} else {
				sound.get_frame_at_position(sound.duration() * relative_time)
			}
		} else {
			Frame::from_mono(0.0)
		}
	}
}
