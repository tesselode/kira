use ringbuf::HeapProducer;

use crate::{parameter::Value, tween::Tween, CommandError, PlaybackRate};

use super::Command;

pub struct SoundHandle {
	pub(crate) command_producer: HeapProducer<Command>,
}

impl SoundHandle {
	pub fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<Value<PlaybackRate>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetPlaybackRate(playback_rate.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
