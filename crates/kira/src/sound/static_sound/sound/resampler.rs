use crate::dsp::{interpolate_frame, Frame};

#[derive(Debug, Clone, Copy, PartialEq)]
struct BufferedFrame {
	frame: Frame,
	position: Option<usize>,
}

pub(super) struct Resampler {
	frames: [BufferedFrame; 4],
	last_frame_position: Option<usize>,
}

impl Resampler {
	pub fn new() -> Self {
		Self {
			frames: [BufferedFrame {
				frame: Frame::ZERO,
				position: None,
			}; 4],
			last_frame_position: None,
		}
	}

	pub fn push_frame(&mut self, frame: Frame, position: impl Into<Option<usize>>) {
		for i in 0..self.frames.len() - 1 {
			self.frames[i] = self.frames[i + 1];
		}
		self.frames[self.frames.len() - 1] = BufferedFrame {
			frame,
			position: position.into(),
		};
		if let Some(position) = self.frames[1].position {
			self.last_frame_position = Some(position);
		}
	}

	pub fn get(&self, fractional_position: f32) -> Frame {
		interpolate_frame(
			self.frames[0].frame,
			self.frames[1].frame,
			self.frames[2].frame,
			self.frames[3].frame,
			fractional_position,
		)
	}

	pub fn position(&self) -> Option<usize> {
		self.last_frame_position
	}

	pub fn is_empty(&self) -> bool {
		self.frames
			.iter()
			.all(|BufferedFrame { frame, .. }| *frame == Frame::ZERO)
	}
}
