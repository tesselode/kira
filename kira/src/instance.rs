//! Provides an interface to control "instances", or individual occurrences, of a sound.
//!
//! You can control the volume and pitch of individual instances as well as pausing, resuming,
//! and stopping them.
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
//! let instance_id = audio_manager.play_sound(sound_id, InstanceSettings::new().pitch(0.5))?;
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
//! # let instance_id = audio_manager.play_sound(sound_id, Default::default())?;
//! audio_manager.stop_instance(instance_id, Some(Tween(2.0)))?;
//! # Ok::<(), Box<dyn Error>>(())
//! ```

use crate::{
	mixer::{SubTrackId, TrackIndex},
	parameter::{Parameter, Parameters, Tween},
	sequence::SequenceId,
	sound::{Sound, SoundId},
	stereo_sample::StereoSample,
	value::CachedValue,
	value::Value,
};
use std::{
	ops::{Range, RangeFrom, RangeFull, RangeTo},
	sync::atomic::{AtomicUsize, Ordering},
};

const MAX_SUB_INSTANCES: usize = 10;

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

/// A track index for an instance to play on.
#[derive(Debug, Copy, Clone)]
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

/// A start or end point of a loop (in seconds).
#[derive(Debug, Copy, Clone)]
pub enum LoopPoint {
	/// The default start or end point.
	Default,
	/// A manually set start or end point.
	Custom(f64),
}

impl LoopPoint {
	fn or_default(&self, default: f64) -> f64 {
		match self {
			LoopPoint::Default => default,
			LoopPoint::Custom(time) => *time,
		}
	}
}

impl Default for LoopPoint {
	fn default() -> Self {
		Self::Default
	}
}

/// A portion of a sound to loop.
#[derive(Debug, Copy, Clone, Default)]
pub struct LoopRegion {
	/// Where the loop starts. Defaults to the beginning of the sound.
	pub start: LoopPoint,
	/// Where the loop ends. Defaults to the semantic duration
	/// of the sound if it's defined, or the very end of the sound
	/// otherwise.
	pub end: LoopPoint,
}

impl LoopRegion {
	fn to_time_range(&self, sound_id: &SoundId) -> Range<f64> {
		(self.start.or_default(0.0))
			..(self.end.or_default(
				sound_id
					.metadata()
					.semantic_duration
					.unwrap_or(sound_id.duration()),
			))
	}
}

impl From<RangeFull> for LoopRegion {
	fn from(_: RangeFull) -> Self {
		Self {
			start: LoopPoint::Default,
			end: LoopPoint::Default,
		}
	}
}

impl From<RangeFrom<f64>> for LoopRegion {
	fn from(range: RangeFrom<f64>) -> Self {
		Self {
			start: LoopPoint::Custom(range.start),
			end: LoopPoint::Default,
		}
	}
}

impl From<RangeTo<f64>> for LoopRegion {
	fn from(range: RangeTo<f64>) -> Self {
		Self {
			start: LoopPoint::Default,
			end: LoopPoint::Custom(range.end),
		}
	}
}

impl From<Range<f64>> for LoopRegion {
	fn from(range: Range<f64>) -> Self {
		Self {
			start: LoopPoint::Custom(range.start),
			end: LoopPoint::Custom(range.end),
		}
	}
}

/// Settings for an instance.
#[derive(Debug, Clone, Copy)]
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
	/// how long the fade-in should last (in seconds).
	pub fade_in_duration: Option<f64>,
	/// Whether the instance should loop, and if so, the region
	/// to loop.
	pub loop_region: Option<LoopRegion>,
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

	/// Sets the amount of time the instance will take to fade in
	/// from silence (in seconds).
	pub fn fade_in_duration(self, fade_in_duration: f64) -> Self {
		Self {
			fade_in_duration: Some(fade_in_duration),
			..self
		}
	}

	/// Sets the portion of the sound that should be looped.
	pub fn loop_region<L: Into<LoopRegion>>(self, region: L) -> Self {
		Self {
			loop_region: Some(region.into()),
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
			fade_in_duration: None,
			loop_region: None,
			track: Default::default(),
		}
	}
}

#[derive(Debug, Clone)]
struct SubInstance {
	position: f64,
	previous_position: f64,
}

impl SubInstance {
	fn new(position: f64) -> Self {
		Self {
			position,
			previous_position: position,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum InstanceState {
	Playing,
	Paused,
	Stopped,
	Resuming,
	Pausing,
	Stopping,
}

#[derive(Debug, Clone)]
pub(crate) struct Instance {
	sound_id: SoundId,
	track_index: TrackIndex,
	sequence_id: Option<SequenceId>,
	volume: CachedValue<f64>,
	pitch: CachedValue<f64>,
	panning: CachedValue<f64>,
	reverse: bool,
	state: InstanceState,
	sub_instances: [Option<SubInstance>; MAX_SUB_INSTANCES],
	next_sub_instance_index: usize,
	fade_volume: Parameter,
	loop_region: Option<Range<f64>>,
}

impl Instance {
	pub fn new(
		sound_id: SoundId,
		sequence_id: Option<SequenceId>,
		settings: InstanceSettings,
	) -> Self {
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
		let mut sub_instances: [Option<SubInstance>; MAX_SUB_INSTANCES] = Default::default();
		sub_instances[0] = Some(SubInstance::new(settings.start_position));
		Self {
			sound_id,
			track_index: settings.track.or_default(sound_id.default_track_index()),
			sequence_id,
			volume: CachedValue::new(settings.volume, 1.0),
			pitch: CachedValue::new(settings.pitch, 1.0),
			panning: CachedValue::new(settings.panning, 0.5),
			reverse: settings.reverse,
			state,
			sub_instances,
			next_sub_instance_index: 1 % MAX_SUB_INSTANCES,
			fade_volume,
			loop_region: settings
				.loop_region
				.map(|region| region.to_time_range(&sound_id)),
		}
	}

	pub fn sound_id(&self) -> SoundId {
		self.sound_id
	}

	pub fn track_index(&self) -> TrackIndex {
		self.track_index
	}

	pub fn sequence_id(&self) -> Option<SequenceId> {
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
			// increment positions of existing sub-instances
			for sub_instance in &mut self.sub_instances {
				if let Some(sub_instance) = sub_instance {
					sub_instance.previous_position = sub_instance.position;
					sub_instance.position += self.pitch.value() * dt;
				}
			}
			// start new sub-instances if previous sub-instances reach the loop point
			if let Some(loop_region) = &self.loop_region {
				for i in 0..self.sub_instances.len() {
					let passed_loop_point = match &self.sub_instances[i] {
						Some(sub_instance) => {
							sub_instance.position >= loop_region.end
								&& sub_instance.previous_position < loop_region.end
						}
						None => false,
					};
					if passed_loop_point {
						self.sub_instances[self.next_sub_instance_index] =
							Some(SubInstance::new(loop_region.start));
						self.next_sub_instance_index += 1;
						self.next_sub_instance_index %= MAX_SUB_INSTANCES;
					}
				}
			}
			// remove finished sub-instances
			for sub_instance in &mut self.sub_instances {
				if let Some(instance) = sub_instance {
					if instance.position >= self.sound_id.duration() || instance.position < 0.0 {
						*sub_instance = None;
					}
				}
			}
			// if all sub-instances are finished, stop the instance
			if self.sub_instances.iter().all(|position| position.is_none()) {
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

	pub fn get_sample(&self, sound: &Sound) -> StereoSample {
		let mut out = StereoSample::from_mono(0.0);
		for sub_instance in &self.sub_instances {
			if let Some(sub_instance) = sub_instance {
				if self.reverse {
					out += sound.get_sample_at_position(sound.duration() - sub_instance.position);
				} else {
					out += sound.get_sample_at_position(sub_instance.position);
				}
			}
		}
		out = out.panned(self.panning.value() as f32);
		out * (self.effective_volume() as f32)
	}
}
