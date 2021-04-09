pub mod handle;

use std::sync::Arc;

use ringbuf::Consumer;

use crate::Frame;

use super::Sound;

pub(crate) const COMMAND_QUEUE_CAPACITY: usize = 10;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Command {
	Pause,
	Resume,
	Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
	Playing,
	Paused,
	Stopped,
}

pub(crate) struct Instance {
	sound: Arc<Sound>,
	state: InstanceState,
	playback_position: f64,
	command_consumer: Consumer<Command>,
}

impl Instance {
	pub fn new(sound: Arc<Sound>, command_consumer: Consumer<Command>) -> Self {
		Self {
			sound,
			state: InstanceState::Playing,
			playback_position: 0.0,
			command_consumer,
		}
	}

	pub fn state(&self) -> InstanceState {
		self.state
	}

	pub fn pause(&mut self) {
		self.state = InstanceState::Paused;
	}

	pub fn resume(&mut self) {
		self.state = InstanceState::Playing;
	}

	pub fn stop(&mut self) {
		self.state = InstanceState::Stopped;
	}

	pub fn process(&mut self, dt: f64) -> Frame {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Pause => self.pause(),
				Command::Resume => self.resume(),
				Command::Stop => self.stop(),
			}
		}
		match self.state {
			InstanceState::Playing => {
				let output = self.sound.get_frame_at_position(self.playback_position);
				self.playback_position += dt;
				if self.playback_position > self.sound.duration() {
					self.state = InstanceState::Stopped;
				}
				output
			}
			_ => Frame::from_mono(0.0),
		}
	}
}
