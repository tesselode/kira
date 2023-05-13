use std::sync::{atomic::Ordering, Arc};

use ringbuf::HeapProducer;

use crate::{
	modulator::ModulatorId,
	tween::{Tween, Value},
	CommandError,
};

use super::{Command, LfoShared};

/// Controls an LFO modulator.
pub struct LfoHandle {
	pub(super) id: ModulatorId,
	pub(super) command_producer: HeapProducer<Command>,
	pub(super) shared: Arc<LfoShared>,
}

impl LfoHandle {
	/// Returns the unique identifier for the modulator.
	pub fn id(&self) -> ModulatorId {
		self.id
	}

	/// Sets how quickly the value oscillates.
	pub fn set_frequency(
		&mut self,
		target: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFrequency {
				target: target.into(),
				tween,
			})
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much the value oscillates.
	///
	/// An amplitude of `2.0` means the modulator will reach a maximum
	/// value of `2.0` and a minimum value of `-2.0`.
	pub fn set_amplitude(
		&mut self,
		target: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetAmplitude {
				target: target.into(),
				tween,
			})
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets a constant value that the modulator is offset by.
	///
	/// An LFO with an offset of `1.0` and an amplitude of `0.5` will reach
	/// a maximum value of `1.5` and a minimum value of `0.5`.
	pub fn set_offset(
		&mut self,
		target: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetOffset {
				target: target.into(),
				tween,
			})
			.map_err(|_| CommandError::CommandQueueFull)
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
