use std::f32::consts::TAU;

use crate::{sound::Sound, Frame};

const INTERNAL_BUFFER_SIZE: usize = 128;

pub struct Renderer {
	dt: f64,
	sound: Sine,
}

impl Renderer {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			sound: Sine::new(),
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
	}

	pub fn on_start_processing(&mut self) {}

	pub fn process(&mut self, out: &mut [f32], num_channels: u16) {
		for chunk in out.chunks_mut(INTERNAL_BUFFER_SIZE * num_channels as usize) {
			self.process_chunk(chunk, num_channels);
		}
	}

	fn process_chunk(&mut self, chunk: &mut [f32], num_channels: u16) {
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		let num_frames = chunk.len() / num_channels as usize;
		self.sound.process(&mut frames[0..num_frames], self.dt);
		for (i, channels) in chunk.chunks_mut(num_channels.into()).enumerate() {
			let frame = frames[i];
			if num_channels == 1 {
				channels[0] = (frame.left + frame.right) / 2.0;
			} else {
				channels[0] = frame.left;
				channels[1] = frame.right;
				/*
					if there's more channels, send silence to them. if we don't,
					we might get bad sounds outputted to those channels.
					(https://github.com/tesselode/kira/issues/50)
				*/
				for channel in channels.iter_mut().skip(2) {
					*channel = 0.0;
				}
			}
		}
	}
}

pub struct Sine {
	phase: f32,
}

impl Sine {
	pub fn new() -> Self {
		Self { phase: 0.0 }
	}
}

impl Sound for Sine {
	fn process(&mut self, out: &mut [Frame], dt: f64) {
		for frame in out {
			*frame = Frame::from_mono(0.1 * (self.phase * TAU).sin());
			self.phase += 440.0 * dt as f32;
			self.phase %= 1.0;
		}
	}

	fn finished(&self) -> bool {
		false
	}
}
