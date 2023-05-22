use std::sync::Arc;

use ringbuf::HeapRb;

use crate::{
	modulator::{Modulator, ModulatorBuilder, ModulatorId},
	tween::Value,
};

use super::{handle::LfoHandle, Lfo, LfoShared, Waveform};

const COMMAND_CAPACITY: usize = 8;

/// Configures an LFO modulator.
#[non_exhaustive]
pub struct LfoBuilder {
	/// The oscillation pattern.
	pub waveform: Waveform,
	/// How quickly the value oscillates.
	pub frequency: Value<f64>,
	/// How much the value oscillates.
	///
	/// An amplitude of `2.0` means the modulator will reach a maximum
	/// value of `2.0` and a minimum value of `-2.0`.
	pub amplitude: Value<f64>,
	/// The constant value the modulator is offset by.
	///
	/// An LFO with an offset of `1.0` and an amplitude of `0.5` will reach
	/// a maximum value of `1.5` and a minimum value of `0.5`.
	pub offset: Value<f64>,
	/// The phase the LFO should start at (in radians).
	///
	/// This determines when in the oscillation the modulator will start.
	pub starting_phase: f64,
}

impl LfoBuilder {
	/// Creates a new [`LfoBuilder`] with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the oscillation pattern.
	pub fn waveform(self, waveform: Waveform) -> Self {
		Self { waveform, ..self }
	}

	/// Sets how quickly the value oscillates.
	pub fn frequency(self, frequency: impl Into<Value<f64>>) -> Self {
		Self {
			frequency: frequency.into(),
			..self
		}
	}

	/// Sets how much the value oscillates.
	///
	/// An amplitude of `2.0` means the modulator will reach a maximum
	/// value of `2.0` and a minimum value of `-2.0`.
	pub fn amplitude(self, amplitude: impl Into<Value<f64>>) -> Self {
		Self {
			amplitude: amplitude.into(),
			..self
		}
	}

	/// Sets a constant value that the modulator is offset by.
	///
	/// An LFO with an offset of `1.0` and an amplitude of `0.5` will reach
	/// a maximum value of `1.5` and a minimum value of `0.5`.
	pub fn offset(self, offset: impl Into<Value<f64>>) -> Self {
		Self {
			offset: offset.into(),
			..self
		}
	}

	/// Sets the phase the LFO should start at (in radians).
	///
	/// This determines when in the oscillation the modulator will start.
	pub fn starting_phase(self, starting_phase: f64) -> Self {
		Self {
			starting_phase,
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
			starting_phase: 0.0,
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
