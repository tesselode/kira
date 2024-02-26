use std::sync::{atomic::Ordering, Arc};

use crate::{
	command::ValueChangeCommand,
	modulator::ModulatorId,
	tween::{Tween, Value},
};

use super::{CommandWriters, LfoShared, Waveform};

/// Controls an LFO modulator.
pub struct LfoHandle {
	pub(super) id: ModulatorId,
	pub(super) shared: Arc<LfoShared>,
	pub(super) command_writers: CommandWriters,
}

impl LfoHandle {
	/// Returns the unique identifier for the modulator.
	pub fn id(&self) -> ModulatorId {
		self.id
	}

	/// Sets the oscillation pattern.
	pub fn set_waveform(&mut self, waveform: Waveform) {
		self.command_writers.waveform_change.write(waveform)
	}

	/// Sets how quickly the value oscillates.
	pub fn set_frequency(&mut self, frequency: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.frequency_change
			.write(ValueChangeCommand {
				target: frequency.into(),
				tween,
			})
	}

	/// Sets how much the value oscillates.
	///
	/// An amplitude of `2.0` means the modulator will reach a maximum
	/// value of `2.0` and a minimum value of `-2.0`.
	pub fn set_amplitude(&mut self, amplitude: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.amplitude_change
			.write(ValueChangeCommand {
				target: amplitude.into(),
				tween,
			})
	}

	/// Sets a constant value that the modulator is offset by.
	///
	/// An LFO with an offset of `1.0` and an amplitude of `0.5` will reach
	/// a maximum value of `1.5` and a minimum value of `0.5`.
	pub fn set_offset(&mut self, offset: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.offset_change
			.write(ValueChangeCommand {
				target: offset.into(),
				tween,
			})
	}

	/// Sets the phase of the LFO (in radians).
	pub fn set_phase(&mut self, phase: f64) {
		self.command_writers.phase_change.write(phase)
	}
}

impl Drop for LfoHandle {
	fn drop(&mut self) {
		self.shared.removed.store(true, Ordering::SeqCst);
	}
}

impl From<&LfoHandle> for ModulatorId {
	fn from(value: &LfoHandle) -> Self {
		value.id
	}
}
