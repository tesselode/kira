pub mod handle;

use std::sync::{atomic::AtomicUsize, Arc};

use atomic::{Atomic, Ordering};
use basedrop::Shared;

use crate::Frame;

use super::Sound;

static NEXT_INSTANCE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstanceId(usize);

impl InstanceId {
	pub(crate) fn new() -> Self {
		Self(NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed))
	}
}

/// Enables two-way communication between an instance and the
/// outside world.
pub(crate) struct InstanceController {
	/// The ID of the instance this controller is meant to control.
	/// The controller might be repurposed for another instance later,
	/// so this is used to make sure the instance doesn't listen to
	/// commands meant for another instance.
	pub instance_id: InstanceId,
	/// The desired playback state of the instance. The instance
	/// will check for changes to this state.
	pub playback_state: Atomic<InstancePlaybackState>,
	/// The playback position of the instance. The instance will update
	/// this so `InstanceHandle`s can report the playback position
	/// back to the user.
	pub playback_position: Atomic<f64>,
}

impl InstanceController {
	pub fn new() -> Self {
		Self {
			instance_id: InstanceId::new(),
			playback_state: Atomic::new(InstancePlaybackState::Playing),
			playback_position: Atomic::new(0.0),
		}
	}

	pub fn pause(&self) {
		self.playback_state
			.store(InstancePlaybackState::Paused, Ordering::Relaxed);
	}

	pub fn resume(&self) {
		self.playback_state
			.store(InstancePlaybackState::Playing, Ordering::Relaxed);
	}

	pub fn stop(&self) {
		self.playback_state
			.store(InstancePlaybackState::Stopped, Ordering::Relaxed);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstancePlaybackState {
	Playing,
	Paused,
	Stopped,
}

pub(crate) struct Instance {
	id: InstanceId,
	sound: Arc<Sound>,
	controller: Shared<InstanceController>,
	playback_state: InstancePlaybackState,
	playback_position: f64,
}

impl Instance {
	pub fn new(sound: Arc<Sound>, controller: Shared<InstanceController>) -> Self {
		let playback_state = controller.playback_state.load(Ordering::Relaxed);
		let playback_position = controller.playback_position.load(Ordering::Relaxed);
		Self {
			id: controller.instance_id,
			sound,
			controller,
			playback_state,
			playback_position,
		}
	}

	pub fn state(&self) -> InstancePlaybackState {
		self.playback_state
	}

	pub fn playback_position(&self) -> f64 {
		self.playback_position
	}

	fn controller(&self) -> Option<&InstanceController> {
		if self.controller.instance_id == self.id {
			Some(&self.controller)
		} else {
			None
		}
	}

	fn set_playback_state(&mut self, state: InstancePlaybackState) {
		self.playback_state = state;
		if let Some(controller) = self.controller() {
			controller.playback_state.store(state, Ordering::Relaxed);
		}
	}

	pub fn pause(&mut self) {
		self.set_playback_state(InstancePlaybackState::Paused);
	}

	pub fn resume(&mut self) {
		self.set_playback_state(InstancePlaybackState::Playing);
	}

	pub fn stop(&mut self) {
		self.set_playback_state(InstancePlaybackState::Stopped);
	}

	pub fn process(&mut self, dt: f64) -> Frame {
		if let Some(controller) = self.controller() {
			self.playback_state = controller.playback_state.load(Ordering::Relaxed);
		}
		match self.playback_state {
			InstancePlaybackState::Playing => {
				let output = self.sound.get_frame_at_position(self.playback_position);
				self.playback_position += dt;
				if let Some(controller) = self.controller() {
					controller
						.playback_position
						.store(self.playback_position, Ordering::Relaxed);
				}
				if self.playback_position > self.sound.duration() {
					self.stop();
				}
				output
			}
			_ => Frame::from_mono(0.0),
		}
	}
}
