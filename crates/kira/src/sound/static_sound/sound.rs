use std::f64::consts::TAU;

use crate::{sound::Sound, Frame, INTERNAL_BUFFER_SIZE};

use super::StaticSoundData;

pub struct StaticSound {
	data: StaticSoundData,
	phase: f64,
}

impl StaticSound {
	pub fn new(data: StaticSoundData) -> Self {
		Self { data, phase: 0.0 }
	}
}

impl Sound for StaticSound {
	fn sample_rate(&self) -> u32 {
		self.data.sample_rate
	}

	fn process(&mut self) -> [Frame; INTERNAL_BUFFER_SIZE] {
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for frame in &mut frames {
			let out = Frame::from_mono(0.25 * (self.phase * TAU).sin() as f32);
			self.phase += 440.0 / self.sample_rate() as f64;
			self.phase %= 1.0;
			*frame = out;
		}
		frames
	}

	fn finished(&self) -> bool {
		false
	}
}
