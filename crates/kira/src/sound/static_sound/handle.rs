use std::{error::Error, fmt::Display, sync::Arc};

use ringbuf::Producer;

use crate::{parameter::Tween, value::Value};

use super::{sound::Shared, Command, PlaybackState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandQueueFull;

impl Display for CommandQueueFull {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot send a command to the sound because the command queue is full")
	}
}

impl Error for CommandQueueFull {}

pub struct StaticSoundHandle {
	pub(super) command_producer: Producer<Command>,
	pub(super) shared: Arc<Shared>,
}

impl StaticSoundHandle {
	pub fn state(&self) -> PlaybackState {
		self.shared.state()
	}

	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	pub fn set_volume(&mut self, volume: impl Into<Value>) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetVolume(volume.into()))
			.map_err(|_| CommandQueueFull)
	}

	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value>,
	) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetPlaybackRate(playback_rate.into()))
			.map_err(|_| CommandQueueFull)
	}

	pub fn set_panning(&mut self, panning: impl Into<Value>) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetPanning(panning.into()))
			.map_err(|_| CommandQueueFull)
	}

	pub fn pause(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Pause(tween))
			.map_err(|_| CommandQueueFull)
	}

	pub fn resume(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Resume(tween))
			.map_err(|_| CommandQueueFull)
	}

	pub fn stop(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Stop(tween))
			.map_err(|_| CommandQueueFull)
	}

	pub fn seek_to(&mut self, position: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SeekTo(position))
			.map_err(|_| CommandQueueFull)
	}

	pub fn seek_by(&mut self, amount: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SeekBy(amount))
			.map_err(|_| CommandQueueFull)
	}
}
