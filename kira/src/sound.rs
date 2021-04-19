//! A chunk of audio data.

use crate::Frame;

use self::data::SoundData;

pub mod data;
pub mod handle;
pub mod instance;
pub mod settings;

pub(crate) struct Sound {
	data: Box<dyn SoundData>,
	loop_start: Option<f64>,
	semantic_duration: Option<f64>,
}

impl Sound {
	pub fn new(
		data: impl SoundData + 'static,
		loop_start: Option<f64>,
		semantic_duration: Option<f64>,
	) -> Self {
		Self {
			data: Box::new(data),
			loop_start,
			semantic_duration,
		}
	}

	pub fn duration(&self) -> f64 {
		self.data.duration()
	}

	pub fn loop_start(&self) -> Option<f64> {
		self.loop_start
	}

	pub fn semantic_duration(&self) -> Option<f64> {
		self.semantic_duration
	}

	pub fn frame_at_position(&self, position: f64) -> Frame {
		self.data.frame_at_position(position)
	}
}
