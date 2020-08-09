use crate::sound::SoundId;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InstanceId {
	index: usize,
}

impl InstanceId {
	pub fn new() -> Self {
		let index = NEXT_INSTANCE_INDEX.fetch_add(1, Ordering::Relaxed);
		Self { index }
	}
}

#[derive(Debug, Copy, Clone)]
pub struct InstanceSettings {
	pub volume: f32,
	pub pitch: f32,
	pub position: f32,
	pub fade_in_duration: Option<f32>,
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self {
			volume: 1.0,
			pitch: 1.0,
			position: 0.0,
			fade_in_duration: None,
		}
	}
}

#[derive(PartialEq)]
pub enum InstanceState {
	Playing,
	Paused,
	Stopped,
	Resuming(f32),
	Pausing(f32),
	Stopping(f32),
}

pub(crate) struct Instance {
	pub sound_id: SoundId,
	duration: f32,
	pub volume: f32,
	pub pitch: f32,
	state: InstanceState,
	position: f32,
	fade_volume: f32,
}

impl Instance {
	pub fn new(sound_id: SoundId, settings: InstanceSettings, duration: f32) -> Self {
		let state;
		let fade_volume;
		if let Some(duration) = settings.fade_in_duration {
			state = InstanceState::Resuming(duration);
			fade_volume = 0.0;
		} else {
			state = InstanceState::Playing;
			fade_volume = 1.0;
		}
		Self {
			sound_id,
			duration,
			volume: settings.volume,
			pitch: settings.pitch,
			state,
			position: settings.position,
			fade_volume,
		}
	}

	pub fn effective_volume(&self) -> f32 {
		self.volume * self.fade_volume
	}

	pub fn position(&self) -> f32 {
		self.position
	}

	pub fn playing(&self) -> bool {
		match self.state {
			InstanceState::Playing => true,
			InstanceState::Paused => false,
			InstanceState::Stopped => false,
			InstanceState::Resuming(_) => true,
			InstanceState::Pausing(_) => true,
			InstanceState::Stopping(_) => true,
		}
	}

	pub fn finished(&self) -> bool {
		self.state == InstanceState::Stopped
	}

	pub fn pause(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Pausing(duration);
		} else {
			self.state = InstanceState::Paused;
		}
	}

	pub fn resume(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Resuming(duration);
		} else {
			self.state = InstanceState::Playing;
		}
	}

	pub fn stop(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Stopping(duration);
		} else {
			self.state = InstanceState::Stopped;
		}
	}

	pub fn update(&mut self, dt: f32) {
		if self.playing() {
			self.position += self.pitch * dt;
			if self.position >= self.duration {
				self.state = InstanceState::Stopped;
			}
		}
		match self.state {
			InstanceState::Resuming(fade_duration) => {
				self.fade_volume += dt / fade_duration;
				if self.fade_volume >= 1.0 {
					self.fade_volume = 1.0;
					self.state = InstanceState::Playing;
				}
			}
			InstanceState::Pausing(fade_duration) => {
				self.fade_volume -= dt / fade_duration;
				if self.fade_volume <= 0.0 {
					self.fade_volume = 0.0;
					self.state = InstanceState::Paused;
				}
			}
			InstanceState::Stopping(fade_duration) => {
				self.fade_volume -= dt / fade_duration;
				if self.fade_volume <= 0.0 {
					self.fade_volume = 0.0;
					self.state = InstanceState::Stopped;
				}
			}
			_ => {}
		}
	}
}
