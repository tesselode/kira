use ringbuf::HeapProducer;

use crate::{parameter::Value, tween::Tween, CommandError};

use super::{Command, FilterMode};

/// Controls a filter effect.
pub struct FilterHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl FilterHandle {
	/// Sets the frequencies that the filter will remove.
	pub fn set_mode(&mut self, mode: FilterMode) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMode(mode))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the cutoff frequency of the filter (in hertz).
	pub fn set_cutoff(
		&mut self,
		cutoff: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetCutoff(cutoff.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the resonance of the filter.
	pub fn set_resonance(
		&mut self,
		resonance: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetResonance(resonance.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn set_mix(
		&mut self,
		mix: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
