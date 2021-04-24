use crate::{
	mixer::track::{handle::TrackHandle, TrackInput},
	sound::Sound,
	value::Value,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceLoopStart {
	DefaultForSound,
	Custom(f64),
	None,
}

impl InstanceLoopStart {
	pub(crate) fn into_option(self, sound: &Sound) -> Option<f64> {
		match self {
			InstanceLoopStart::DefaultForSound => sound.loop_start(),
			InstanceLoopStart::Custom(loop_start) => Some(loop_start),
			InstanceLoopStart::None => None,
		}
	}
}

impl Default for InstanceLoopStart {
	fn default() -> Self {
		Self::DefaultForSound
	}
}

impl From<f64> for InstanceLoopStart {
	fn from(loop_start: f64) -> Self {
		Self::Custom(loop_start)
	}
}

#[derive(Clone)]
pub struct InstanceSettings {
	pub(crate) volume: Value<f64>,
	pub(crate) playback_rate: Value<f64>,
	pub(crate) panning: Value<f64>,
	pub(crate) reverse: bool,
	pub(crate) start_position: f64,
	pub(crate) loop_start: InstanceLoopStart,
	pub(crate) track: Option<TrackInput>,
}

impl InstanceSettings {
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(1.0),
			playback_rate: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			reverse: false,
			start_position: 0.0,
			loop_start: InstanceLoopStart::default(),
			track: None,
		}
	}

	pub fn volume(self, volume: impl Into<Value<f64>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	pub fn playback_rate(self, playback_rate: impl Into<Value<f64>>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	pub fn panning(self, panning: impl Into<Value<f64>>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	pub fn reverse(self, reverse: bool) -> Self {
		Self { reverse, ..self }
	}

	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	pub fn loop_start(self, loop_start: impl Into<InstanceLoopStart>) -> Self {
		Self {
			loop_start: loop_start.into(),
			..self
		}
	}

	pub fn track(self, track: &TrackHandle) -> Self {
		Self {
			track: Some(track.input()),
			..self
		}
	}

	pub(crate) fn into_internal(
		&self,
		sound: &Sound,
		main_track_input: TrackInput,
	) -> InternalInstanceSettings {
		let start_position = if self.reverse {
			sound.duration() - self.start_position
		} else {
			self.start_position
		};
		InternalInstanceSettings {
			volume: self.volume.clone(),
			playback_rate: self.playback_rate.clone(),
			panning: self.panning.clone(),
			reverse: self.reverse,
			start_position,
			loop_start: self.loop_start.into_option(sound),
			track: if let Some(track) = &self.track {
				track.clone()
			} else {
				main_track_input.clone()
			},
		}
	}
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone)]
pub(crate) struct InternalInstanceSettings {
	pub(crate) volume: Value<f64>,
	pub(crate) playback_rate: Value<f64>,
	pub(crate) panning: Value<f64>,
	pub(crate) reverse: bool,
	pub(crate) start_position: f64,
	pub(crate) loop_start: Option<f64>,
	pub(crate) track: TrackInput,
}
