//! Individual occurrences of sounds.

mod handle;
mod settings;

pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicU64, AtomicU8, Ordering},
	Arc,
};

use atomic_arena::Index;

use crate::{
	clock::ClockTime,
	frame::Frame,
	loop_behavior::LoopBehavior,
	manager::resources::{clocks::Clocks, mixer::Mixer, sounds::Sounds, Parameters},
	parameter::{Parameter, Tween},
	start_time::StartTime,
	track::TrackId,
	value::{cached::CachedValue, Value},
};

use super::{wrapper::SoundWrapper, Sound, SoundId};

/// A unique identifier for an instance of a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub(crate) Index);

/// The playback state of an instance of a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceState {
	/// The instance is playing.
	Playing,
	/// The instance is fading out, and when the fade-out
	/// is finished, the instance will pause playback.
	Pausing,
	/// The instance is paused.
	Paused,
	/// The instance is fading out, and when the fade-out
	/// is finished, the instance will stop.
	Stopping,
	/// The instance is stopped and cannot be interacted with
	/// further.
	Stopped,
}

impl InstanceState {
	fn from_u8(value: u8) -> Self {
		match value {
			0 => Self::Playing,
			1 => Self::Pausing,
			2 => Self::Paused,
			3 => Self::Stopping,
			4 => Self::Stopped,
			_ => panic!("{} is not a valid InstanceState", value),
		}
	}

	fn is_playing(&self) -> bool {
		matches!(
			self,
			InstanceState::Playing | InstanceState::Pausing | InstanceState::Stopping
		)
	}
}

pub(crate) struct InstanceShared {
	state: AtomicU8,
	position: AtomicU64,
}

impl InstanceShared {
	pub fn state(&self) -> InstanceState {
		InstanceState::from_u8(self.state.load(Ordering::SeqCst))
	}

	pub fn position(&self) -> f64 {
		f64::from_bits(self.position.load(Ordering::SeqCst))
	}
}

pub(crate) struct Instance {
	sound_id: SoundId,
	track: TrackId,
	start_time: StartTime,
	waiting_to_start: bool,
	volume: CachedValue,
	playback_rate: CachedValue,
	panning: CachedValue,
	reverse: bool,
	loop_behavior: Option<LoopBehavior>,
	state: InstanceState,
	position: f64,
	fade_volume: Parameter,
	shared: Arc<InstanceShared>,
}

impl Instance {
	pub fn new(sound_id: SoundId, sound: &Arc<dyn Sound>, settings: InstanceSettings) -> Self {
		let position = if settings.reverse {
			sound.duration().as_secs_f64() - settings.start_position
		} else {
			settings.start_position
		};
		Self {
			sound_id,
			track: settings.track,
			start_time: settings.start_time,
			waiting_to_start: matches!(settings.start_time, StartTime::ClockTime(..)),
			volume: CachedValue::new(.., settings.volume, 1.0),
			playback_rate: CachedValue::new(.., settings.playback_rate, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
			reverse: settings.reverse,
			loop_behavior: settings.loop_behavior.as_option(sound),
			state: InstanceState::Playing,
			position,
			fade_volume: if let Some(tween) = settings.fade_in_tween {
				let mut parameter = Parameter::new(0.0);
				parameter.set(1.0, tween);
				parameter
			} else {
				Parameter::new(1.0)
			},
			shared: Arc::new(InstanceShared {
				state: AtomicU8::new(InstanceState::Playing as u8),
				position: AtomicU64::new(position.to_bits()),
			}),
		}
	}

	pub fn shared(&self) -> Arc<InstanceShared> {
		self.shared.clone()
	}

	pub fn state(&self) -> InstanceState {
		self.state
	}

	fn set_state(&mut self, state: InstanceState) {
		self.state = state;
		self.shared.state.store(state as u8, Ordering::SeqCst);
	}

	pub fn set_volume(&mut self, volume: Value) {
		self.volume.set(volume);
	}

	pub fn set_playback_rate(&mut self, playback_rate: Value) {
		self.playback_rate.set(playback_rate);
	}

	pub fn set_panning(&mut self, panning: Value) {
		self.panning.set(panning);
	}

	pub fn pause(&mut self, fade_out_tween: Tween) {
		if self.waiting_to_start && matches!(fade_out_tween.start_time, StartTime::Immediate) {
			self.set_state(InstanceState::Paused);
			return;
		}
		self.set_state(InstanceState::Pausing);
		self.fade_volume.set(0.0, fade_out_tween);
	}

	pub fn resume(&mut self, fade_in_tween: Tween) {
		self.set_state(InstanceState::Playing);
		self.fade_volume.set(1.0, fade_in_tween);
	}

	pub fn stop(&mut self, fade_out_tween: Tween) {
		if self.waiting_to_start && matches!(fade_out_tween.start_time, StartTime::Immediate) {
			self.set_state(InstanceState::Stopped);
			return;
		}
		self.set_state(InstanceState::Stopping);
		self.fade_volume.set(0.0, fade_out_tween);
	}

	pub fn seek_to(&mut self, position: f64) {
		self.position = position;
	}

	pub fn seek_by(&mut self, amount: f64) {
		self.position += amount;
	}

	pub fn on_start_processing(&self) {
		self.shared
			.position
			.store(self.position.to_bits(), Ordering::SeqCst);
	}

	pub fn process(
		&mut self,
		dt: f64,
		sounds: &Sounds,
		parameters: &Parameters,
		clocks: &Clocks,
		mixer: &mut Mixer,
	) {
		if self.waiting_to_start {
			if let StartTime::ClockTime(ClockTime { clock, ticks }) = self.start_time {
				if let Some(clock) = clocks.get(clock) {
					if clock.ticking() && clock.ticks() >= ticks {
						self.waiting_to_start = false;
					}
				}
			} else {
				panic!("waiting_to_start should always be false if the start_time is Immediate");
			}
		}
		if self.waiting_to_start {
			return;
		}
		if let Some(track) = mixer.track_mut(self.track) {
			track.add_input(self.get_output(dt, sounds, parameters, clocks));
		}
	}

	fn get_output(
		&mut self,
		dt: f64,
		sounds: &Sounds,
		parameters: &Parameters,
		clocks: &Clocks,
	) -> Frame {
		let sound = match sounds.get(self.sound_id) {
			Some(sound) => sound,
			None => return Frame::from_mono(0.0),
		};
		if self.state.is_playing() {
			self.volume.update(parameters);
			self.playback_rate.update(parameters);
			self.panning.update(parameters);
			let just_finished_fade = self.fade_volume.update(dt, clocks);
			let out = sound
				.sound
				.frame_at_position(self.position)
				.panned(self.panning.get() as f32)
				* self.volume.get() as f32
				* self.fade_volume.value() as f32;
			self.update_playback_position(dt, sound);
			if just_finished_fade {
				match self.state {
					InstanceState::Pausing => {
						self.set_state(InstanceState::Paused);
					}
					InstanceState::Stopping => {
						self.set_state(InstanceState::Stopped);
					}
					_ => {}
				}
			}
			return out;
		}
		Frame::from_mono(0.0)
	}

	fn update_playback_position(&mut self, dt: f64, sound: &SoundWrapper) {
		let playback_rate = if self.reverse {
			-self.playback_rate.get()
		} else {
			self.playback_rate.get()
		};
		self.position += playback_rate * dt;
		let duration = sound.sound.duration().as_secs_f64();
		if playback_rate < 0.0 {
			if let Some(loop_behavior) = self.loop_behavior {
				while self.position < loop_behavior.start_position {
					self.position += duration - loop_behavior.start_position;
				}
			} else if self.position < 0.0 {
				self.set_state(InstanceState::Stopped);
			}
		} else {
			if let Some(loop_behavior) = self.loop_behavior {
				while self.position > duration {
					self.position -= duration - loop_behavior.start_position;
				}
			} else if self.position > duration {
				self.set_state(InstanceState::Stopped);
			}
		}
	}
}
