//! A chunk of audio data.

use crate::Frame;

use self::data::SoundData;

pub mod data;
pub mod handle;
pub mod instance;

pub(crate) struct Sound {
	data: Box<dyn SoundData>,
}

impl Sound {
	pub fn new(data: impl SoundData + 'static) -> Self {
		Self {
			data: Box::new(data),
		}
	}

	pub fn duration(&self) -> f64 {
		self.data.duration()
	}

	pub fn frame_at_position(&self, position: f64) -> Frame {
		self.data.frame_at_position(position)
	}
}
