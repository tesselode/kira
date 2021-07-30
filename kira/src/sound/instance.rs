pub mod handle;
pub mod settings;

use std::sync::{
	atomic::{AtomicU64, AtomicU8, Ordering},
	Arc,
};

use atomic_arena::Index;

use crate::{
	frame::Frame,
	manager::{
		backend::context::Context,
		resources::{mixer::Mixer, parameters::Parameters, sounds::Sounds},
	},
	parameter::{tween::Tween, Parameter},
	track::TrackId,
	value::{cached::CachedValue, Value},
};

use self::settings::InstanceSettings;

use super::{data::SoundData, Sound, SoundId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub(crate) Index);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceState {
	Playing,
	Pausing,
	Paused,
	Stopping,
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
		match self {
			InstanceState::Playing | InstanceState::Pausing | InstanceState::Stopping => true,
			_ => false,
		}
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
	time_until_start: f64,
	volume: CachedValue,
	playback_rate: CachedValue,
	panning: CachedValue,
	reverse: bool,
	loop_start: Option<f64>,
	state: InstanceState,
	position: f64,
	fade_volume: Parameter,
	shared: Arc<InstanceShared>,
}

impl Instance {
	pub fn new(
		sound_id: SoundId,
		sound_data: &Arc<dyn SoundData>,
		settings: InstanceSettings,
	) -> Self {
		let position = if settings.reverse {
			sound_data.duration().as_secs_f64() - settings.start_position
		} else {
			settings.start_position
		};
		Self {
			sound_id,
			track: settings.track,
			time_until_start: settings.delay.as_secs_f64(),
			volume: CachedValue::new(.., settings.volume, 1.0),
			playback_rate: CachedValue::new(.., settings.playback_rate, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
			reverse: settings.reverse,
			loop_start: settings.loop_start.as_option(sound_data),
			state: InstanceState::Playing,
			position,
			fade_volume: Parameter::new(1.0),
			shared: Arc::new(InstanceShared {
				state: AtomicU8::new(InstanceState::Playing as u8),
				position: AtomicU64::new(position.to_bits()),
			}),
		}
	}

	pub fn reduce_delay_time(&mut self, time: f64) {
		self.time_until_start -= time;
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

	pub fn pause(&mut self, tween: Tween, context: &Arc<Context>, command_sent_time: u64) {
		if self.time_until_start > 0.0 {
			self.set_state(InstanceState::Paused);
			return;
		}
		self.set_state(InstanceState::Pausing);
		self.fade_volume.set(context, 0.0, tween, command_sent_time);
	}

	pub fn resume(&mut self, tween: Tween, context: &Arc<Context>, command_sent_time: u64) {
		self.set_state(InstanceState::Playing);
		self.fade_volume.set(context, 1.0, tween, command_sent_time);
	}

	pub fn stop(&mut self, tween: Tween, context: &Arc<Context>, command_sent_time: u64) {
		if self.time_until_start > 0.0 {
			self.set_state(InstanceState::Stopped);
			return;
		}
		self.set_state(InstanceState::Stopping);
		self.fade_volume.set(context, 0.0, tween, command_sent_time);
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
		mixer: &mut Mixer,
	) {
		if let Some(track) = mixer.track_mut(self.track) {
			track.add_input(self.get_output(dt, sounds, parameters));
		}
	}

	fn get_output(&mut self, dt: f64, sounds: &Sounds, parameters: &Parameters) -> Frame {
		let sound = match sounds.get(self.sound_id) {
			Some(sound) => sound,
			None => return Frame::from_mono(0.0),
		};
		if self.state.is_playing() {
			if self.time_until_start > 0.0 {
				self.time_until_start -= dt;
				return Frame::from_mono(0.0);
			}
			self.volume.update(parameters);
			self.playback_rate.update(parameters);
			self.panning.update(parameters);
			let just_finished_fade = self.fade_volume.update(dt);
			let out = sound
				.data
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

	fn update_playback_position(&mut self, dt: f64, sound: &Sound) {
		let playback_rate = if self.reverse {
			-self.playback_rate.get()
		} else {
			self.playback_rate.get()
		};
		self.position += playback_rate * dt;
		let duration = sound.data.duration().as_secs_f64();
		if playback_rate < 0.0 {
			if let Some(loop_start) = self.loop_start {
				while self.position < loop_start {
					self.position += duration - loop_start;
				}
			} else if self.position < 0.0 {
				self.set_state(InstanceState::Stopped);
			}
		} else {
			if let Some(loop_start) = self.loop_start {
				while self.position > duration {
					self.position -= duration - loop_start;
				}
			} else if self.position > duration {
				self.set_state(InstanceState::Stopped);
			}
		}
	}
}
