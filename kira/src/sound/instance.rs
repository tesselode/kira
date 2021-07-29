pub mod handle;
pub mod settings;

use std::sync::Arc;

use atomic_arena::Index;

use crate::{
	frame::Frame,
	manager::{
		backend::context::Context,
		resources::{parameters::Parameters, sounds::Sounds},
	},
	parameter::{tween::Tween, Parameter},
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
	fn is_playing(&self) -> bool {
		match self {
			InstanceState::Playing | InstanceState::Pausing | InstanceState::Stopping => true,
			_ => false,
		}
	}
}

pub(crate) struct Instance {
	sound_id: SoundId,
	start_time: u64,
	volume: CachedValue,
	playback_rate: CachedValue,
	panning: CachedValue,
	reverse: bool,
	loop_start: Option<f64>,
	state: InstanceState,
	position: f64,
	fade_volume: Parameter,
}

impl Instance {
	pub fn new(
		context: &Arc<Context>,
		sound_id: SoundId,
		sound_data: &Arc<dyn SoundData>,
		settings: InstanceSettings,
	) -> Self {
		Self {
			sound_id,
			start_time: context.sample_count()
				+ ((settings.delay.as_secs_f64() * context.sample_rate() as f64) as u64),
			volume: CachedValue::new(.., settings.volume, 1.0),
			playback_rate: CachedValue::new(.., settings.playback_rate, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
			reverse: settings.reverse,
			loop_start: settings.loop_start.as_option(sound_data),
			state: InstanceState::Playing,
			position: if settings.reverse {
				sound_data.duration().as_secs_f64() - settings.start_position
			} else {
				settings.start_position
			},
			fade_volume: Parameter::new(1.0),
		}
	}

	pub fn state(&self) -> InstanceState {
		self.state
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
		self.state = InstanceState::Pausing;
		self.fade_volume
			.tween(context, 0.0, tween, command_sent_time);
	}

	pub fn resume(&mut self, tween: Tween, context: &Arc<Context>, command_sent_time: u64) {
		self.state = InstanceState::Playing;
		self.fade_volume
			.tween(context, 1.0, tween, command_sent_time);
	}

	pub fn stop(&mut self, tween: Tween, context: &Arc<Context>, command_sent_time: u64) {
		self.state = InstanceState::Stopping;
		self.fade_volume
			.tween(context, 0.0, tween, command_sent_time);
	}

	pub fn process(
		&mut self,
		sample_count: u64,
		dt: f64,
		sounds: &Sounds,
		parameters: &Parameters,
	) -> Frame {
		if sample_count < self.start_time {
			return Frame::from_mono(0.0);
		}
		let sound = match sounds.get(self.sound_id) {
			Some(sound) => sound,
			None => return Frame::from_mono(0.0),
		};
		if self.state.is_playing() {
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
						self.state = InstanceState::Paused;
					}
					InstanceState::Stopping => {
						self.state = InstanceState::Stopped;
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
				self.state = InstanceState::Stopped;
			}
		} else {
			if let Some(loop_start) = self.loop_start {
				while self.position > duration {
					self.position -= duration - loop_start;
				}
			} else if self.position > duration {
				self.state = InstanceState::Stopped;
			}
		}
	}
}
