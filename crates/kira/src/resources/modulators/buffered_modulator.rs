use std::ops::{Deref, DerefMut};

use arrayvec::ArrayVec;

use crate::{info::SingleFrameInfo, modulator::Modulator, INTERNAL_BUFFER_SIZE};

#[derive(Default)]
pub(crate) struct BufferedModulator {
	modulator: Box<dyn Modulator>,
	value_buffer: ArrayVec<f64, INTERNAL_BUFFER_SIZE>,
}

impl BufferedModulator {
	pub fn new(modulator: Box<dyn Modulator>) -> Self {
		Self {
			modulator,
			value_buffer: ArrayVec::new(),
		}
	}

	pub fn value_buffer(&self) -> &[f64] {
		&self.value_buffer
	}

	pub fn update(&mut self, dt: f64, info: &SingleFrameInfo) {
		self.modulator.update(dt, info);
		self.value_buffer.push(self.modulator.value());
	}

	pub fn clear_buffer(&mut self) {
		self.value_buffer.clear();
	}
}

impl Default for Box<dyn Modulator> {
	fn default() -> Self {
		Box::new(DummyModulator)
	}
}

impl Deref for BufferedModulator {
	type Target = Box<dyn Modulator>;

	fn deref(&self) -> &Self::Target {
		&self.modulator
	}
}

impl DerefMut for BufferedModulator {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.modulator
	}
}

struct DummyModulator;

impl Modulator for DummyModulator {
	fn update(&mut self, _dt: f64, _info: &SingleFrameInfo) {}

	fn value(&self) -> f64 {
		0.0
	}

	fn finished(&self) -> bool {
		false
	}
}
