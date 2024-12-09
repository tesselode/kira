use crate::{interpolate_frame, Frame, INTERNAL_BUFFER_SIZE};

use super::Sound;

pub struct SoundWrapper {
	sound: Box<dyn Sound>,
	chunk: [Frame; INTERNAL_BUFFER_SIZE],
	chunk_frame_index: usize,
	resample_buffer: [Frame; 4],
	fractional_position: f64,
}

impl SoundWrapper {
	pub fn new(sound: Box<dyn Sound>) -> Self {
		Self {
			sound,
			chunk: [Frame::ZERO; INTERNAL_BUFFER_SIZE],
			chunk_frame_index: 0,
			resample_buffer: [Frame::ZERO; 4],
			fractional_position: 0.0,
		}
	}

	pub fn on_start_processing(&mut self) {
		self.sound.on_start_processing();
	}

	pub fn process(&mut self, dt: f64) -> [Frame; INTERNAL_BUFFER_SIZE] {
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for frame in &mut frames {
			*frame = self.interpolated_frame();
			self.fractional_position += self.sound.sample_rate() as f64 * dt;
			while self.fractional_position >= 1.0 {
				self.fractional_position -= 1.0;
				let next_frame = self.next_frame();
				self.push_to_resample_buffer(next_frame);
			}
		}
		frames
	}

	pub fn finished(&self) -> bool {
		self.sound.finished() && self.outputting_silence()
	}

	#[must_use]
	fn outputting_silence(&self) -> bool {
		self.resample_buffer
			.iter()
			.all(|frame| *frame == Frame::ZERO)
	}

	#[must_use]
	pub fn interpolated_frame(&self) -> Frame {
		interpolate_frame(
			self.resample_buffer[0],
			self.resample_buffer[1],
			self.resample_buffer[2],
			self.resample_buffer[3],
			self.fractional_position as f32,
		)
	}

	fn push_to_resample_buffer(&mut self, frame: Frame) {
		for i in 0..self.resample_buffer.len() - 1 {
			self.resample_buffer[i] = self.resample_buffer[i + 1];
		}
		self.resample_buffer[self.resample_buffer.len() - 1] = frame;
	}

	fn next_frame(&mut self) -> Frame {
		if self.chunk_frame_index == 0 {
			self.chunk = self.sound.process();
		}
		let frame = self.chunk[self.chunk_frame_index];
		self.chunk_frame_index += 1;
		self.chunk_frame_index %= INTERNAL_BUFFER_SIZE;
		frame
	}
}
