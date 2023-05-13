use std::sync::{atomic::Ordering, Arc};

use ringbuf::HeapProducer;

use crate::{
	modulator::ModulatorId,
	tween::{Tween, Value},
	CommandError,
};

use super::{Command, LfoShared};

pub struct LfoHandle {
	pub(super) id: ModulatorId,
	pub(super) command_producer: HeapProducer<Command>,
	pub(super) shared: Arc<LfoShared>,
}

impl LfoHandle {
	pub fn id(&self) -> ModulatorId {
		self.id
	}

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
