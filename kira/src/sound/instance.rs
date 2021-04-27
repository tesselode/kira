pub mod handle;
pub mod settings;

use std::sync::{atomic::AtomicUsize, Arc};

use atomig::{Atom, Atomic, Ordering};
use basedrop::Shared;

use crate::{mixer::track::TrackInput, value::Value};

use self::settings::InternalInstanceSettings;

use super::Sound;

static NEXT_INSTANCE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Atom)]
#[repr(u8)]
pub enum InstancePlaybackState {
	Playing,
	Paused,
	Stopped,
}

pub(crate) struct InstanceController {
	id: AtomicUsize,
	playback_state: Atomic<InstancePlaybackState>,
	playback_position: Atomic<f64>,
}

impl InstanceController {
	pub fn new() -> Self {
		#[cfg(feature = "log_drops")]
		println!(
			"creating InstanceController on thread {:?}",
			std::thread::current().id()
		);
		Self {
			id: AtomicUsize::new(NEXT_INSTANCE_ID.fetch_add(1, Ordering::SeqCst)),
			playback_state: Atomic::new(InstancePlaybackState::Playing),
			playback_position: Atomic::new(0.0),
		}
	}

	pub fn with_settings(settings: &InternalInstanceSettings) -> Self {
		#[cfg(feature = "log_drops")]
		println!(
			"creating InstanceController on thread {:?}",
			std::thread::current().id()
		);
		Self {
			id: AtomicUsize::new(NEXT_INSTANCE_ID.fetch_add(1, Ordering::SeqCst)),
			playback_state: Atomic::new(InstancePlaybackState::Playing),
			playback_position: Atomic::new(settings.start_position),
		}
	}

	pub fn reinit_with_settings(&self, settings: &InternalInstanceSettings) {
		self.id.store(
			NEXT_INSTANCE_ID.fetch_add(1, Ordering::SeqCst),
			Ordering::SeqCst,
		);
		self.playback_state
			.store(InstancePlaybackState::Playing, Ordering::SeqCst);
		self.playback_position
			.store(settings.start_position, Ordering::SeqCst);
	}

	pub fn playback_state(&self) -> InstancePlaybackState {
		self.playback_state.load(Ordering::SeqCst)
	}

	pub fn playback_position(&self) -> f64 {
		self.playback_position.load(Ordering::SeqCst)
	}

	pub fn pause(&self) {
		self.playback_state
			.store(InstancePlaybackState::Paused, Ordering::SeqCst);
	}

	pub fn resume(&self) {
		self.playback_state
			.store(InstancePlaybackState::Playing, Ordering::SeqCst);
	}

	pub fn stop(&self) {
		self.playback_state
			.store(InstancePlaybackState::Stopped, Ordering::SeqCst);
	}
}

#[cfg(feature = "log_drops")]
impl Drop for InstanceController {
	fn drop(&mut self) {
		println!(
			"dropped InstanceController on thread {:?}",
			std::thread::current().id()
		);
	}
}

// TODO: wrap these values in an Owned or something.
// eventually it will become possible to remove parameters,
// in which case the instance could be the last thing with
// an Arc<Parameter>. when it drops, it will deallocate memory
// on the audio thread
pub(crate) struct Instance {
	id: usize,
	sound: Arc<Sound>,
	controller: Shared<Arc<InstanceController>>,
	playback_state: InstancePlaybackState,
	playback_position: f64,
	volume: Value<f64>,
	playback_rate: Value<f64>,
	panning: Value<f64>,
	reverse: bool,
	loop_start: Option<f64>,
	output_dest: TrackInput,
}

impl Instance {
	pub fn new(
		sound: Arc<Sound>,
		controller: Shared<Arc<InstanceController>>,
		settings: InternalInstanceSettings,
	) -> Self {
		let playback_state = controller.playback_state();
		let playback_position = controller.playback_position();
		Self {
			id: controller.id.load(Ordering::SeqCst),
			sound,
			controller,
			playback_state,
			playback_position,
			volume: settings.volume,
			playback_rate: settings.playback_rate,
			panning: settings.panning,
			reverse: settings.reverse,
			loop_start: settings.loop_start,
			output_dest: settings.track,
		}
	}

	pub fn sound(&self) -> &Arc<Sound> {
		&self.sound
	}

	pub fn playback_state(&self) -> InstancePlaybackState {
		self.playback_state
	}

	pub fn playback_position(&self) -> f64 {
		self.playback_position
	}

	fn set_playback_state(&mut self, state: InstancePlaybackState) {
		self.playback_state = state;
		if self.controller.id.load(Ordering::SeqCst) == self.id {
			self.controller
				.playback_state
				.store(state, Ordering::SeqCst);
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

	pub fn process(&mut self, dt: f64) {
		if self.controller.id.load(Ordering::SeqCst) == self.id {
			self.playback_state = self.controller.playback_state();
		}
		if let InstancePlaybackState::Playing = self.playback_state() {
			let output = self.sound.frame_at_position(self.playback_position);
			let mut playback_rate = self.playback_rate.get();
			if self.reverse {
				playback_rate *= -1.0;
			}
			self.playback_position += playback_rate * dt;
			if playback_rate < 0.0 {
				if let Some(loop_start) = self.loop_start {
					while self.playback_position < loop_start {
						self.playback_position += self.sound.duration() - loop_start;
					}
				} else if self.playback_position < 0.0 {
					self.stop();
				}
			} else {
				if let Some(loop_start) = self.loop_start {
					while self.playback_position > self.sound.duration() {
						self.playback_position -= self.sound.duration() - loop_start;
					}
				} else if self.playback_position > self.sound.duration() {
					self.stop();
				}
			}
			self.output_dest
				.add(output.panned(self.panning.get() as f32) * self.volume.get() as f32);
		}
		if self.controller.id.load(Ordering::SeqCst) == self.id {
			self.controller
				.playback_position
				.store(self.playback_position, Ordering::SeqCst);
		}
	}
}
