use std::time::Duration;

use ringbuf::HeapProducer;

use crate::{parameter::Value, tween::Tween, CommandError};

use super::Command;

pub struct CompressorHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl CompressorHandle {
	pub fn set_threshold(
		&mut self,
		threshold: impl Into<Value<f32>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetThreshold(threshold.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_ratio(
		&mut self,
		ratio: impl Into<Value<f32>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetRatio(ratio.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_attack_duration(
		&mut self,
		attack_duration: impl Into<Value<Duration>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetAttackDuration(attack_duration.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	pub fn set_release_duration(
		&mut self,
		release_duration: impl Into<Value<Duration>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetReleaseDuration(release_duration.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
