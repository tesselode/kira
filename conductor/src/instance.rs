/*!
Contains structs related to instances.

Each time you play a sound, it creates an "instance", or occurrence, of that sound.
Each instance can be controlled independently. Multiple instances of the same sound
can be playing at once.
*/

use crate::{parameter::Parameter, sound::SoundId, tween::Tween};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(0);

/**
A unique identifier for an `Instance`.

You cannot create this manually - an `InstanceId` is created
when you play a sound with an `AudioManager`.
*/
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

/// Settings for an instance.
#[derive(Debug, Copy, Clone)]
pub struct InstanceSettings {
	/// The volume of the instance.
	pub volume: f32,
	/// The pitch of the instance, as a factor of the original pitch.
	pub pitch: f32,
	/// The position to start playing the instance at (in seconds).
	pub position: f32,
	/// Whether to fade in the instance from silence, and if so,
	/// how long the fade-in should last (in seconds).
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
pub(crate) enum InstanceState {
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
	pub volume: Parameter,
	pub pitch: Parameter,
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
			fade_volume.set(1.0, Some(Tween(duration)));
		} else {
			state = InstanceState::Playing;
			fade_volume = Parameter::new(1.0);
		}
		Self {
			sound_id,
			duration,
			volume: Parameter::new(settings.volume),
			pitch: Parameter::new(settings.pitch),
			state,
			position: settings.position,
			fade_volume,
		}
	}

	pub fn effective_volume(&self) -> f32 {
		self.volume.value() * self.fade_volume.value()
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

	pub fn set_volume(&mut self, volume: f32, tween: Option<Tween>) {
		self.volume.set(volume, tween);
	}

	pub fn set_pitch(&mut self, pitch: f32, tween: Option<Tween>) {
		self.pitch.set(pitch, tween);
	}

	pub fn pause(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Pausing;
			self.fade_volume.set(0.0, Some(Tween(duration)));
		} else {
			self.state = InstanceState::Paused;
		}
	}

	pub fn resume(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Resuming;
			self.fade_volume.set(1.0, Some(Tween(duration)));
		} else {
			self.state = InstanceState::Playing;
		}
	}

	pub fn stop(&mut self, fade_duration: Option<f32>) {
		if let Some(duration) = fade_duration {
			self.state = InstanceState::Stopping;
			self.fade_volume.set(0.0, Some(Tween(duration)));
		} else {
			self.state = InstanceState::Stopped;
		}
	}

	pub fn update(&mut self, dt: f32) {
		if self.playing() {
			self.volume.update(dt);
			self.pitch.update(dt);
			self.position += self.pitch.value() * dt;
			if self.position >= self.duration {
				self.state = InstanceState::Stopped;
			}
		}
		let finished_fading = self.fade_volume.update(dt);
		if finished_fading {
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
