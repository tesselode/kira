use crate::{parameter::Parameter, sound::SoundId};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InstanceId {
	index: usize,
}

impl InstanceId {
	pub(crate) fn new() -> Self {
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
	Resuming,
	Pausing,
	Stopping,
}

pub(crate) struct Instance {
	pub sound_id: SoundId,
	duration: f32,
	pub volume: f32,
	pub pitch: f32,
	state: InstanceState,
	position: f32,
	fade_volume: Parameter,
}

impl Instance {
	pub fn new(sound_id: SoundId, settings: InstanceSettings, duration: f32) -> Self {
		let state;
		let mut fade_volume;
		if let Some(duration) = settings.fade_in_duration {
			state = InstanceState::Resuming;
			fade_volume = Parameter::new(0.0);
			fade_volume.tween(1.0, duration);
		} else {
			state = InstanceState::Playing;
			fade_volume = Parameter::new(1.0);
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
		self.volume * self.fade_volume.value()
	}

	pub fn position(&self) -> f32 {
		self.position
	}

	pub fn playing(&self) -> bool {
		match self.state {
			InstanceState::Playing => true,
			InstanceState::Paused => false,
			InstanceState::Stopped => false,
			InstanceState::Resuming => true,
			InstanceState::Pausing => true,
			InstanceState::Stopping => true,
		}
	}

	pub fn finished(&self) -> bool {
		self.state == InstanceState::Stopped
	}

	pub fn pause(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Pausing;
			self.fade_volume.tween(0.0, duration);
		} else {
			self.state = InstanceState::Paused;
		}
	}

	pub fn resume(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Resuming;
			self.fade_volume.tween(1.0, duration);
		} else {
			self.state = InstanceState::Playing;
		}
	}

	pub fn stop(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Stopping;
			self.fade_volume.tween(0.0, duration);
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
		let finished = self.fade_volume.update(dt);
		if finished {
			match self.state {
				InstanceState::Resuming => {
					self.state = InstanceState::Playing;
				}
				InstanceState::Pausing => {
					self.state = InstanceState::Paused;
				}
				InstanceState::Stopping => {
					self.state = InstanceState::Stopped;
				}
				_ => {}
			}
		}
	}
}
