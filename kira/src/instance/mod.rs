//! Provides an interface to control "instances", or individual occurrences,
//! of [`Sound`](crate::sound::Sound)s and [`Arrangement`](crate::arrangement::Arrangement)s.
//!
//! You can control the volume and pitch of individual instances as well as
//! pausing, resuming, and stopping them.
//!
//! ## Examples
//!
//! ### Playing a sound at a lower pitch than normal
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{manager::AudioManager, instance::InstanceSettings, sound::Sound};
//! #
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! let instance_id = audio_manager.play(sound_id, InstanceSettings::new().pitch(0.5))?;
//! # Ok::<(), Box<dyn Error>>(())
//! ```
//!
//! ### Fading out a sound over 2 seconds
//!
//! ```no_run
//! # use std::error::Error;
//! #
//! # use kira::{manager::AudioManager, sound::Sound, parameter::Tween, instance::StopInstanceSettings};
//! #
//! # let mut audio_manager = AudioManager::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let instance_id = audio_manager.play(sound_id, Default::default())?;
//! audio_manager.stop_instance(instance_id, StopInstanceSettings::new().fade_tween(Some(2.0.into())))?;
//! # Ok::<(), Box<dyn Error>>(())
//! ```
//!
//! ## Reverse playback and loop points
//!
//! There are two ways to enable reverse playback:
//! - Enabling the reverse setting
//! - Setting the pitch of the instance to a negative number
//!
//! Enabling the reverse setting also adjusts the instance's
//! starting position to be relative to the end of the sound,
//! while setting the pitch to a negative number doesn't. In
//! 99% of cases, if you want an instance to play backwards,
//! you should use the reverse flag.
//!
//! You can get some interesting effects by tweening a pitch
//! from a positive to a negative number and vice versa, so
//! there's still some value to using negative pitches.
//!
//! If you have the reverse playback enabled *and* the pitch
//! is negative, you will end up with forward playback.
//!
//! If the instance has a loop start point and it's playing
//! backward, when the playback position is earlier than the
//! loop start point, it will wrap around to the end
//! of the instance.

pub mod handle;
mod settings;

use atomic::Atomic;
use handle::InstanceHandle;
pub use settings::*;

use uuid::Uuid;

use crate::{
	frame::Frame,
	mixer::TrackIndex,
	parameter::{Parameter, Parameters},
	playable::{PlayableId, Playables},
	sequence::SequenceInstanceId,
	util::generate_uuid,
	value::CachedValue,
	value::Value,
};
use std::sync::{atomic::Ordering, Arc};

/**
A unique identifier for an instance.

You cannot create this manually - an instance ID is created
when you play a sound with an [`AudioManager`](crate::manager::AudioManager).
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct InstanceId {
	uuid: Uuid,
}

impl InstanceId {
	pub(crate) fn new() -> Self {
		Self {
			uuid: generate_uuid(),
		}
	}
}

impl From<&InstanceHandle> for InstanceId {
	fn from(handle: &InstanceHandle) -> Self {
		handle.id()
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InstanceState {
	Playing,
	Paused(f64),
	Stopped,
	Pausing(f64),
	Stopping,
}

#[derive(Debug, Clone)]
pub(crate) struct Instance {
	playable_id: PlayableId,
	duration: f64,
	sequence_id: Option<SequenceInstanceId>,
	track_index: TrackIndex,
	volume: CachedValue<f64>,
	pitch: CachedValue<f64>,
	panning: CachedValue<f64>,
	reverse: bool,
	loop_start: Option<f64>,
	state: InstanceState,
	public_state: Arc<Atomic<InstanceState>>,
	position: f64,
	fade_volume: Parameter,
}

impl Instance {
	pub fn new(
		playable: PlayableId,
		duration: f64,
		sequence_id: Option<SequenceInstanceId>,
		settings: InternalInstanceSettings,
	) -> Self {
		let mut fade_volume;
		if let Some(tween) = settings.fade_in_tween {
			fade_volume = Parameter::new(0.0);
			fade_volume.set(1.0, Some(tween));
		} else {
			fade_volume = Parameter::new(1.0);
		}
		Self {
			playable_id: playable,
			duration,
			sequence_id,
			track_index: settings.track,
			volume: CachedValue::new(settings.volume, 1.0),
			pitch: CachedValue::new(settings.pitch, 1.0),
			panning: CachedValue::new(settings.panning, 0.5),
			reverse: settings.reverse,
			loop_start: settings.loop_start,
			state: InstanceState::Playing,
			public_state: Arc::new(Atomic::new(InstanceState::Playing)),
			position: settings.start_position,
			fade_volume,
		}
	}

	pub fn playable_id(&self) -> PlayableId {
		self.playable_id
	}

	pub fn track_index(&self) -> TrackIndex {
		self.track_index
	}

	pub fn sequence_id(&self) -> Option<SequenceInstanceId> {
		self.sequence_id
	}

	pub fn effective_volume(&self) -> f64 {
		self.volume.value() * self.fade_volume.value()
	}

	pub fn public_state(&self) -> Arc<Atomic<InstanceState>> {
		self.public_state.clone()
	}

	pub fn playing(&self) -> bool {
		match self.state {
			InstanceState::Playing => true,
			InstanceState::Paused(_) => false,
			InstanceState::Stopped => false,
			InstanceState::Pausing(_) => true,
			InstanceState::Stopping => true,
		}
	}

	pub fn finished(&self) -> bool {
		self.state == InstanceState::Stopped
	}

	pub fn set_volume(&mut self, volume: Value<f64>) {
		self.volume.set(volume);
	}

	pub fn set_pitch(&mut self, pitch: Value<f64>) {
		self.pitch.set(pitch);
	}

	pub fn set_panning(&mut self, panning: Value<f64>) {
		self.panning.set(panning);
	}

	pub fn seek(&mut self, offset: f64) {
		self.position += offset;
	}

	pub fn seek_to(&mut self, position: f64) {
		self.position = position;
	}

	fn set_state(&mut self, state: InstanceState) {
		self.state = state;
		self.public_state.store(state, Ordering::Relaxed);
	}

	pub fn pause(&mut self, settings: PauseInstanceSettings) {
		self.set_state(if settings.fade_tween.is_some() {
			InstanceState::Pausing(self.position)
		} else {
			InstanceState::Paused(self.position)
		});
		self.fade_volume.set(0.0, settings.fade_tween);
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) {
		match self.state {
			InstanceState::Paused(position) | InstanceState::Pausing(position) => {
				self.set_state(InstanceState::Playing);
				if settings.rewind_to_pause_position {
					self.seek_to(position);
				}
				self.fade_volume.set(1.0, settings.fade_tween);
			}
			_ => {}
		}
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) {
		self.set_state(if settings.fade_tween.is_some() {
			InstanceState::Stopping
		} else {
			InstanceState::Stopped
		});
		self.fade_volume.set(0.0, settings.fade_tween);
	}

	pub fn update(&mut self, dt: f64, parameters: &Parameters) {
		if self.playing() {
			self.volume.update(parameters);
			self.pitch.update(parameters);
			self.panning.update(parameters);
			let mut pitch = self.pitch.value();
			if self.reverse {
				pitch *= -1.0;
			}
			self.position += pitch * dt;
			if pitch < 0.0 {
				if let Some(loop_start) = self.loop_start {
					while self.position < loop_start {
						self.position += self.duration - loop_start;
					}
				} else if self.position < 0.0 {
					self.set_state(InstanceState::Stopped);
				}
			} else {
				if let Some(loop_start) = self.loop_start {
					while self.position > self.duration {
						self.position -= self.duration - loop_start;
					}
				} else if self.position > self.duration {
					self.set_state(InstanceState::Stopped);
				}
			}
		}
		let finished_fading = self.fade_volume.update(dt);
		if finished_fading {
			match self.state {
				InstanceState::Pausing(position) => {
					self.set_state(InstanceState::Paused(position));
				}
				InstanceState::Stopping => {
					self.set_state(InstanceState::Stopped);
				}
				_ => {}
			}
		}
	}

	pub fn get_sample(&self, playables: &Playables) -> Frame {
		let mut out = playables
			.frame_at_position(self.playable_id, self.position)
			.unwrap_or(Frame::from_mono(0.0));
		out = out.panned(self.panning.value() as f32);
		out * (self.effective_volume() as f32)
	}
}
