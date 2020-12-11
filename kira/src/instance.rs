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

use indexmap::IndexMap;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	frame::Frame,
	mixer::{SubTrackId, TrackIndex},
	parameter::{Parameter, Parameters, Tween},
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

/// A track index for an instance to play on.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InstanceTrackIndex {
	/// The default track for the sound.
	DefaultForSound,
	/// A manually set track index.
	Custom(TrackIndex),
}

impl InstanceTrackIndex {
	fn or_default(&self, default: TrackIndex) -> TrackIndex {
		match self {
			InstanceTrackIndex::DefaultForSound => default,
			InstanceTrackIndex::Custom(index) => *index,
		}
	}
}

impl Default for InstanceTrackIndex {
	fn default() -> Self {
		Self::DefaultForSound
	}
}

impl From<TrackIndex> for InstanceTrackIndex {
	fn from(index: TrackIndex) -> Self {
		Self::Custom(index)
	}
}

impl From<SubTrackId> for InstanceTrackIndex {
	fn from(id: SubTrackId) -> Self {
		Self::Custom(TrackIndex::Sub(id))
	}
}

/// A loop start point for an instance.
#[derive(Debug, Copy, Clone)]
pub enum InstanceLoopStart {
	Default,
	None,
	Custom(f64),
}

impl InstanceLoopStart {
	fn into_option(&self, playable: Playable) -> Option<f64> {
		match self {
			Self::Default => playable.default_loop_start(),
			Self::None => None,
			Self::Custom(position) => Some(*position),
		}
	}
}

impl Default for InstanceLoopStart {
	fn default() -> Self {
		Self::Default
	}
}

impl From<f64> for InstanceLoopStart {
	fn from(position: f64) -> Self {
		Self::Custom(position)
	}
}

impl From<Option<f64>> for InstanceLoopStart {
	fn from(option: Option<f64>) -> Self {
		match option {
			Some(position) => Self::Custom(position),
			None => Self::None,
		}
	}
}

/// Settings for an instance.
#[derive(Debug, Copy, Clone)]
pub struct InstanceSettings {
	/// The volume of the instance.
	pub volume: Value<f64>,
	/// The pitch of the instance, as a factor of the original pitch.
	pub pitch: Value<f64>,
	/// The panning of the instance (0 = hard left, 1 = hard right).
	pub panning: Value<f64>,
	/// Whether the instance should be played in reverse.
	pub reverse: bool,
	/// The position to start playing the instance at (in seconds).
	pub start_position: f64,
	/// Whether to fade in the instance from silence, and if so,
	/// the tween to use.
	pub fade_in_tween: Option<Tween>,
	/// Whether the instance should loop, and if so, the position
	/// it should jump back to when it reaches the end.
	pub loop_start: InstanceLoopStart,
	/// Which track to play the instance on.
	pub track: InstanceTrackIndex,
}

impl InstanceSettings {
	/// Creates a new `InstanceSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the volume of the instance.
	pub fn volume<V: Into<Value<f64>>>(self, volume: V) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the pitch of the instance.
	pub fn pitch<P: Into<Value<f64>>>(self, pitch: P) -> Self {
		Self {
			pitch: pitch.into(),
			..self
		}
	}

	/// Sets the panning of the instance.
	pub fn panning<P: Into<Value<f64>>>(self, panning: P) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	/// Enables reverse playback for the instance.
	pub fn reverse(self) -> Self {
		Self {
			reverse: true,
			..self
		}
	}

	/// Sets where in the sound playback will start (in seconds).
	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	/// Sets the tween the instance will use to fade in from silence.
	pub fn fade_in_tween(self, fade_in_tween: Tween) -> Self {
		Self {
			fade_in_tween: Some(fade_in_tween),
			..self
		}
	}

	/// Sets the portion of the sound that should be looped.
	pub fn loop_start<S: Into<InstanceLoopStart>>(self, start: S) -> Self {
		Self {
			loop_start: start.into(),
			..self
		}
	}

	/// Sets the track the instance will play on.
	pub fn track<T: Into<InstanceTrackIndex>>(self, track: T) -> Self {
		Self {
			track: track.into(),
			..self
		}
	}
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			pitch: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			reverse: false,
			start_position: 0.0,
			fade_in_tween: None,
			loop_start: InstanceLoopStart::default(),
			track: InstanceTrackIndex::default(),
		}
	}
}

// TODO: remove unnecessary states
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum InstanceState {
	Playing,
	Paused,
	Stopped,
	Resuming,
	Pausing,
	Stopping,
}

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
		let state;
		let mut fade_volume;
		if let Some(tween) = settings.fade_in_tween {
			state = InstanceState::Resuming;
			fade_volume = Parameter::new(0.0);
			fade_volume.set(1.0, Some(tween));
		} else {
			state = InstanceState::Playing;
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
			state,
			position: 0.0,
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

	pub fn set_volume(&mut self, volume: Value<f64>) {
		self.volume.set(volume);
	}

	pub fn set_pitch(&mut self, pitch: Value<f64>) {
		self.pitch.set(pitch);
	}

	pub fn set_panning(&mut self, panning: Value<f64>) {
		self.panning.set(panning);
	}

	pub fn pause(&mut self, fade_tween: Option<Tween>) {
		if let Some(tween) = fade_tween {
			self.state = InstanceState::Pausing;
			self.fade_volume.set(0.0, Some(tween));
		} else {
			self.state = InstanceState::Paused;
		}
	}

	pub fn resume(&mut self, fade_tween: Option<Tween>) {
		if let Some(tween) = fade_tween {
			self.state = InstanceState::Resuming;
			self.fade_volume.set(1.0, Some(tween));
		} else {
			self.state = InstanceState::Playing;
		}
	}

	pub fn stop(&mut self, fade_tween: Option<Tween>) {
		if let Some(tween) = fade_tween {
			self.state = InstanceState::Stopping;
			self.fade_volume.set(0.0, Some(tween));
		} else {
			self.state = InstanceState::Stopped;
		}
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
