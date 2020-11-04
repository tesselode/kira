/*!
Contains structs related to instances.

Each time you play a sound, it creates an "instance", or occurrence, of that sound.
Each instance can be controlled independently. Multiple instances of the same sound
can be playing at once.
*/

use crate::{
	manager::backend::parameters::Parameters,
	parameter::Parameter,
	sequence::SequenceId,
	sound::{Sound, SoundId},
	stereo_sample::StereoSample,
	track::index::TrackIndex,
	tween::Tween,
	value::CachedValue,
	value::Value,
};
use std::{
	ops::Range,
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

/// Settings for how an instance should loop.
#[derive(Debug, Clone, Default)]
pub struct LoopSettings {
	/// Where the loop starts. Defaults to the beginning of the sound.
	pub start: Option<f64>,
	/// Where the loop ends. Defaults to the semantic duration
	/// of the sound if it's defined, or the very end of the sound
	/// otherwise.
	pub end: Option<f64>,
}

/// Settings for an instance.
#[derive(Debug, Clone)]
pub struct InstanceSettings {
	/// The volume of the instance.
	pub volume: Value,
	/// The pitch of the instance, as a factor of the original pitch.
	pub pitch: Value,
	/// Whether the instance should be played in reverse.
	pub reverse: bool,
	/// The position to start playing the instance at (in seconds).
	pub position: f64,
	/// Whether to fade in the instance from silence, and if so,
	/// how long the fade-in should last (in seconds).
	pub fade_in_duration: Option<f64>,
	/// Whether the instance should loop, and if so, settings
	/// for how it should loop.
	pub loop_settings: Option<LoopSettings>,
	/// Which track to play the instance on. Defaults to the
	/// sound's default track.
	pub track: Option<TrackIndex>,
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			pitch: Value::Fixed(1.0),
			reverse: false,
			position: 0.0,
			fade_in_duration: None,
			loop_settings: None,
			track: None,
		}
	}
}

impl From<()> for InstanceSettings {
    fn from(_: ()) -> Self {
        Self::default()
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
	volume: CachedValue,
	pitch: CachedValue,
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
		sub_instances[0] = Some(SubInstance::new(settings.position));
		Self {
			sound_id,
			track_index: settings.track.unwrap_or(sound_id.default_track_index()),
			sequence_id,
			volume: CachedValue::new(settings.volume, 1.0),
			pitch: CachedValue::new(settings.pitch, 1.0),
			reverse: settings.reverse,
			state,
			sub_instances,
			next_sub_instance_index: 1 % MAX_SUB_INSTANCES,
			fade_volume,
			loop_region: match settings.loop_settings {
				Some(loop_settings) => Some(
					loop_settings.start.unwrap_or(0.0)
						..loop_settings.end.unwrap_or(
							sound_id
								.metadata()
								.semantic_duration
								.unwrap_or(sound_id.duration()),
						),
				),
				None => None,
			},
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

	pub fn set_volume(&mut self, volume: Value) {
		self.volume.set(volume);
	}

	pub fn set_pitch(&mut self, pitch: Value) {
		self.pitch.set(pitch);
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
		out * (self.effective_volume() as f32)
	}
}
