//! Provides an interface to control "instances", or individual occurrences,
//! of a [`Sound`](crate::sound::Sound).
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
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
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
//! # use kira::{manager::AudioManager, sound::Sound, parameter::Tween};
//! #
//! # let mut audio_manager = AudioManager::<()>::new(Default::default())?;
//! # let sound_id = audio_manager.add_sound(Sound::from_file("loop.ogg", Default::default())?)?;
//! # let instance_id = audio_manager.play(sound_id, Default::default())?;
//! audio_manager.stop_instance(instance_id, Some(2.0.into()))?;
//! # Ok::<(), Box<dyn Error>>(())
//! ```

mod settings;

pub use settings::*;

use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	frame::Frame,
	group::{groups::Groups, GroupId},
	mixer::TrackIndex,
	parameter::{Parameter, Parameters},
	playable::Playable,
	sequence::SequenceInstanceId,
	sound::{Sound, SoundId},
	value::CachedValue,
	value::Value,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(0);

/**
A unique identifier for an instance.

You cannot create this manually - an instance ID is created
when you play a sound with an [`AudioManager`](crate::manager::AudioManager).
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum InstanceState {
	Playing,
	Paused(f64),
	Stopped,
	Pausing(f64),
	Stopping,
}

/*
TODO: make sure all the looping behavior is good

Scenarios worth considering:
- Forward playback
	- Seeking forward past the end of the sound - playback
	position should wrap around like normal
	- Seeking backward past the loop start point - playback
	should continue moving forward
- Forward playback with negative pitch (effectively reverse playback)
	- Crossed loop start point - should wrap around to end
	- Seeking forward past the end of the sound - ???
	- Seeking backward past the loop start - should wrap around to end

Setting the reverse flag is not actually equivalent to starting
from the end of the sound and playing with negative pitch,
but it probably should be to simplify behavior. Currently, setting
the reverse flag causes the instance to read the sound/arrangement
backwards, which means the loop points will be different relative
to the content of the sound. I don't think this is a behavior
you would ever actually want, although I could be wrong.
*/

#[derive(Debug, Copy, Clone)]
pub(crate) struct Instance {
	playable: Playable,
	track_index: TrackIndex,
	sequence_id: Option<SequenceInstanceId>,
	volume: CachedValue<f64>,
	pitch: CachedValue<f64>,
	panning: CachedValue<f64>,
	loop_start: Option<f64>,
	reverse: bool,
	state: InstanceState,
	position: f64,
	fade_volume: Parameter,
}

impl Instance {
	pub fn new(
		playable: Playable,
		sequence_id: Option<SequenceInstanceId>,
		settings: InstanceSettings,
	) -> Self {
		let mut fade_volume;
		if let Some(tween) = settings.fade_in_tween {
			fade_volume = Parameter::new(0.0);
			fade_volume.set(1.0, Some(tween));
		} else {
			fade_volume = Parameter::new(1.0);
		}
		Self {
			playable,
			track_index: settings.track.or_default(playable.default_track()),
			sequence_id,
			volume: CachedValue::new(settings.volume, 1.0),
			pitch: CachedValue::new(settings.pitch, 1.0),
			panning: CachedValue::new(settings.panning, 0.5),
			reverse: settings.reverse,
			loop_start: settings.loop_start.into_option(playable),
			state: InstanceState::Playing,
			position: settings.start_position,
			fade_volume,
		}
	}

	pub fn playable(&self) -> Playable {
		self.playable
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

	pub fn is_in_group(
		&self,
		parent_id: GroupId,
		sounds: &IndexMap<SoundId, Sound>,
		arrangements: &IndexMap<ArrangementId, Arrangement>,
		groups: &Groups,
	) -> bool {
		self.playable
			.is_in_group(parent_id, sounds, arrangements, groups)
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

	pub fn pause(&mut self, settings: PauseInstanceSettings) {
		self.state = if settings.fade_tween.is_some() {
			InstanceState::Pausing(self.position)
		} else {
			InstanceState::Paused(self.position)
		};
		self.fade_volume.set(0.0, settings.fade_tween);
	}

	pub fn resume(&mut self, settings: ResumeInstanceSettings) {
		match self.state {
			InstanceState::Paused(position) | InstanceState::Pausing(position) => {
				self.state = InstanceState::Playing;
				if settings.rewind_to_pause_position {
					self.seek_to(position);
				}
				self.fade_volume.set(1.0, settings.fade_tween);
			}
			_ => {}
		}
	}

	pub fn stop(&mut self, settings: StopInstanceSettings) {
		self.state = if settings.fade_tween.is_some() {
			InstanceState::Stopping
		} else {
			InstanceState::Stopped
		};
		self.fade_volume.set(0.0, settings.fade_tween);
	}

	pub fn update(&mut self, dt: f64, parameters: &Parameters) {
		if self.playing() {
			self.volume.update(parameters);
			self.pitch.update(parameters);
			self.panning.update(parameters);
			self.position += self.pitch.value() * dt;
			if self.position > self.playable.duration() {
				if let Some(loop_start) = self.loop_start {
					self.position -= self.playable.duration() - loop_start;
				} else {
					self.state = InstanceState::Stopped;
				}
			}
		}
		let finished_fading = self.fade_volume.update(dt);
		if finished_fading {
			match self.state {
				InstanceState::Pausing(position) => {
					self.state = InstanceState::Paused(position);
				}
				InstanceState::Stopping => {
					self.state = InstanceState::Stopped;
				}
				_ => {}
			}
		}
	}

	pub fn get_sample(
		&self,
		sounds: &IndexMap<SoundId, Sound>,
		arrangements: &IndexMap<ArrangementId, Arrangement>,
	) -> Frame {
		let position = if self.reverse {
			self.playable.duration() - self.position
		} else {
			self.position
		};
		let mut out = self
			.playable
			.get_frame_at_position(position, sounds, arrangements);
		out = out.panned(self.panning.value() as f32);
		out * (self.effective_volume() as f32)
	}
}
