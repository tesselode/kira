use std::ops::{Deref, DerefMut};

use crate::{modulator::Modulator, INTERNAL_BUFFER_SIZE};

pub(crate) struct BufferedModulator {
	modulator: Box<dyn Modulator>,
	value_buffer: [f64; INTERNAL_BUFFER_SIZE],
	current_frame: usize,
}

impl BufferedModulator {
	pub fn new(modulator: Box<dyn Modulator>) -> Self {
		Self {
			modulator,
			value_buffer: [0.0; INTERNAL_BUFFER_SIZE],
			current_frame: 0,
		}
	}

	pub fn value_buffer(&self) -> [f64; INTERNAL_BUFFER_SIZE] {
		self.value_buffer
	}

	pub fn update(&mut self, dt: f64) {
		self.modulator.update(dt);
		self.value_buffer[self.current_frame] = self.modulator.value();
		self.current_frame += 1;
	}

	pub fn reset_buffer(&mut self) {
		self.value_buffer = [0.0; INTERNAL_BUFFER_SIZE];
		self.current_frame = 0;
	}
}

impl Default for BufferedModulator {
	fn default() -> Self {
		Self {
			modulator: Default::default(),
			value_buffer: [0.0; INTERNAL_BUFFER_SIZE],
			current_frame: Default::default(),
		}
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
	fn update(&mut self, _dt: f64) {}

	fn value(&self) -> f64 {
		0.0
	}

	fn finished(&self) -> bool {
		false
	}
}
