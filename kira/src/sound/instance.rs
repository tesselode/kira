pub mod handle;
pub mod settings;

use std::sync::Arc;

use atomig::{Atom, Atomic, Ordering};

use crate::{mixer::track::TrackInput, value::Value};

use self::settings::InternalInstanceSettings;

use super::Sound;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Atom)]
#[repr(u8)]
pub enum InstancePlaybackState {
	Playing,
	Paused,
	Stopped,
}

pub(crate) struct Instance {
	playback_state: Atomic<InstancePlaybackState>,
	playback_position: Atomic<f64>,
	volume: Value<f64>,
	playback_rate: Value<f64>,
	panning: Value<f64>,
	reverse: bool,
	loop_start: Option<f64>,
	sound: Arc<Sound>,
	output_dest: TrackInput,
}

impl Instance {
	pub fn new(sound: Arc<Sound>, settings: InternalInstanceSettings) -> Self {
		#[cfg(feature = "log_drops")]
		println!(
			"creating Instance on thread {:?}",
			std::thread::current().id()
		);
		Self {
			sound,
			volume: settings.volume,
			playback_rate: settings.playback_rate,
			panning: settings.panning,
			reverse: settings.reverse,
			loop_start: settings.loop_start,
			playback_state: Atomic::new(InstancePlaybackState::Playing),
			playback_position: Atomic::new(settings.start_position),
			output_dest: settings.track,
		}
	}

	pub fn sound(&self) -> &Arc<Sound> {
		&self.sound
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
			let mut playback_rate = self.playback_rate.get();
			if self.reverse {
				playback_rate *= -1.0;
			}
			playback_position += playback_rate * dt;
			if playback_rate < 0.0 {
				if let Some(loop_start) = self.loop_start {
					while playback_position < loop_start {
						playback_position += self.sound.duration() - loop_start;
					}
				} else if playback_position < 0.0 {
					self.stop();
				}
			} else {
				if let Some(loop_start) = self.loop_start {
					while playback_position > self.sound.duration() {
						playback_position -= self.sound.duration() - loop_start;
					}
				} else if playback_position > self.sound.duration() {
					self.stop();
				}
			}
			self.playback_position
				.store(playback_position, Ordering::SeqCst);
			self.output_dest
				.add(output.panned(self.panning.get() as f32) * self.volume.get() as f32);
		}
	}
}

#[cfg(feature = "log_drops")]
impl Drop for Instance {
	fn drop(&mut self) {
		println!(
			"dropped Instance on thread {:?}",
			std::thread::current().id()
		);
	}
}
