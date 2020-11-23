mod clip;

pub use clip::SoundClip;
use indexmap::IndexMap;

use crate::{
	sound::{Sound, SoundId},
	Frame,
};

#[derive(Debug, Clone)]
pub struct Arrangement {
	clips: Vec<SoundClip>,
	duration: f64,
}

impl Arrangement {
	pub fn new() -> Self {
		Self {
			clips: vec![],
			duration: 0.0,
		}
	}

	pub fn add_clip(mut self, clip: SoundClip) -> Self {
		self.duration = self.duration.max(clip.clip_time_range.end);
		self.clips.push(clip);
		self
	}

	pub fn duration(&self) -> f64 {
		self.duration
	}

	pub(crate) fn get_frame_at_position(
		&self,
		position: f64,
		sounds: &IndexMap<SoundId, Sound>,
	) -> Frame {
		let mut frame = Frame::from_mono(0.0);
		for clip in &self.clips {
			frame += clip.get_frame_at_position(position, sounds);
		}
		frame
	}
}

impl From<SoundId> for Arrangement {
	fn from(id: SoundId) -> Self {
		Self::new().add_clip(SoundClip::new(id, 0.0))
	}
}
