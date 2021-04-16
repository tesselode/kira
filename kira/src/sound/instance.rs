pub mod handle;
pub mod settings;

use atomig::{Atom, Atomic, Ordering};
use basedrop::Shared;

use crate::mixer::track::TrackInput;

use super::Sound;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Atom)]
#[repr(u8)]
pub enum InstancePlaybackState {
	Playing,
	Paused,
	Stopped,
}

pub(crate) struct Instance {
	pub playback_state: Atomic<InstancePlaybackState>,
	pub playback_position: Atomic<f64>,
	sound: Shared<Sound>,
	output_dest: TrackInput,
}

impl Instance {
	pub fn new(sound: Shared<Sound>, output_dest: TrackInput) -> Self {
		Self {
			sound,
			playback_state: Atomic::new(InstancePlaybackState::Playing),
			playback_position: Atomic::new(0.0),
			output_dest,
		}
	}

	pub fn playback_state(&self) -> InstancePlaybackState {
		self.playback_state.load(Ordering::SeqCst)
	}

	pub fn playback_position(&self) -> f64 {
		self.playback_position.load(Ordering::SeqCst)
	}

	fn set_playback_state(&self, state: InstancePlaybackState) {
		self.playback_state.store(state, Ordering::SeqCst);
	}

	pub fn pause(&self) {
		self.set_playback_state(InstancePlaybackState::Paused);
	}

	pub fn resume(&self) {
		self.set_playback_state(InstancePlaybackState::Playing);
	}

	pub fn stop(&self) {
		self.set_playback_state(InstancePlaybackState::Stopped);
	}

	pub fn process(&self, dt: f64) {
		if let InstancePlaybackState::Playing = self.playback_state() {
			let mut playback_position = self.playback_position();
			let output = self.sound.frame_at_position(playback_position);
			playback_position += dt;
			self.playback_position
				.store(playback_position, Ordering::SeqCst);
			if playback_position > self.sound.duration() {
				self.stop();
			}
			self.output_dest.add(output);
		}
	}
}
