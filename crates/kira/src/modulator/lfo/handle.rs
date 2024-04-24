use std::sync::{atomic::Ordering, Arc};

use crate::{handle_param_setters, modulator::ModulatorId};

use super::{CommandWriters, LfoShared, Waveform};

/// Controls an LFO modulator.
pub struct LfoHandle {
	pub(super) id: ModulatorId,
	pub(super) command_writers: CommandWriters,
	pub(super) shared: Arc<LfoShared>,
}

impl LfoHandle {
	/// Returns the unique identifier for the modulator.
	#[must_use]
	pub fn id(&self) -> ModulatorId {
		self.id
	}

	/// Sets the oscillation pattern.
	pub fn set_waveform(&mut self, waveform: Waveform) {
		self.command_writers.set_waveform.write(waveform)
	}

	handle_param_setters! {
		/// Sets how quickly the value oscillates.
		frequency: f64,

		/// Sets how much the value oscillates.
		///
		/// An amplitude of `2.0` means the modulator will reach a maximum
		/// value of `2.0` and a minimum value of `-2.0`.
		amplitude: f64,

		/// Sets a constant value that the modulator is offset by.
		///
		/// An LFO with an offset of `1.0` and an amplitude of `0.5` will reach
		/// a maximum value of `1.5` and a minimum value of `0.5`.
		offset: f64,
	}

	/// Sets the phase of the LFO (in radians).
	pub fn set_phase(&mut self, phase: f64) {
		self.command_writers.set_phase.write(phase)
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
