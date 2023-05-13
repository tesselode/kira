use std::sync::Arc;

use ringbuf::HeapRb;

use crate::{
	modulator::{Modulator, ModulatorBuilder, ModulatorId},
	tween::Value,
};

use super::{handle::LfoHandle, Lfo, LfoShared, Waveform};

const COMMAND_CAPACITY: usize = 8;

pub struct LfoBuilder {
	pub waveform: Waveform,
	pub frequency: Value<f64>,
	pub amplitude: Value<f64>,
	pub offset: Value<f64>,
}

impl LfoBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn waveform(self, waveform: Waveform) -> Self {
		Self { waveform, ..self }
	}

	pub fn frequency(self, frequency: impl Into<Value<f64>>) -> Self {
		Self {
			frequency: frequency.into(),
			..self
		}
	}

	pub fn amplitude(self, amplitude: impl Into<Value<f64>>) -> Self {
		Self {
			amplitude: amplitude.into(),
			..self
		}
	}

	pub fn offset(self, offset: impl Into<Value<f64>>) -> Self {
		Self {
			offset: offset.into(),
			..self
		}
	}
}

impl Default for LfoBuilder {
	fn default() -> Self {
		Self {
			waveform: Waveform::Sine,
			frequency: Value::Fixed(2.0),
			amplitude: Value::Fixed(1.0),
			offset: Value::Fixed(0.0),
		}
	}
}

impl ModulatorBuilder for LfoBuilder {
	type Handle = LfoHandle;

	fn build(self, id: ModulatorId) -> (Box<dyn Modulator>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		let shared = Arc::new(LfoShared::new());
		(
			Box::new(Lfo::new(&self, command_consumer, shared.clone())),
			LfoHandle {
				id,
				command_producer,
				shared,
			},
		)
	}
}
