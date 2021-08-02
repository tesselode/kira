use std::sync::Arc;

use crate::{
	parameter::Tween, sound::data::SoundData, start_time::StartTime, track::TrackId, value::Value,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceLoopStart {
	DefaultForSoundData,
	Custom(f64),
	None,
}

impl InstanceLoopStart {
	pub(crate) fn as_option(self, data: &Arc<dyn SoundData>) -> Option<f64> {
		match self {
			Self::DefaultForSoundData => data.default_loop_start(),
			Self::Custom(loop_start) => Some(loop_start),
			Self::None => None,
		}
	}
}

impl<T: Into<Option<f64>>> From<T> for InstanceLoopStart {
	fn from(loop_start: T) -> Self {
		match loop_start.into() {
			Some(loop_start) => Self::Custom(loop_start),
			None => Self::None,
		}
	}
}

impl Default for InstanceLoopStart {
	fn default() -> Self {
		Self::DefaultForSoundData
	}
}

/// Settings for an instance of a sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceSettings {
	/// When the instance should start playing.
	pub start_time: StartTime,
	/// The initial playback position of the instance (in seconds).
	pub start_position: f64,
	/// The volume of the instance.
	pub volume: Value,
	/// The playback rate of the instance, as a factor of the
	/// normal playback rate.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub playback_rate: Value,
	/// The panning of the instance, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value,
	/// Whether the instance should play in reverse.
	///
	/// If set to `true`, the start position will be relative
	/// to the end of the sound.
	pub reverse: bool,
	/// The looping behavior of the instance.
	pub loop_start: InstanceLoopStart,
	/// The mixer track this instance should play on.
	pub track: TrackId,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}

impl InstanceSettings {
	/// Creates a new [`InstanceSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			start_time: StartTime::default(),
			start_position: 0.0,
			volume: Value::Fixed(1.0),
			playback_rate: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			reverse: false,
			loop_start: InstanceLoopStart::default(),
			track: TrackId::Main,
			fade_in_tween: None,
		}
	}

	/// Sets when the instance should start playing.
	pub fn start_time(self, start_time: StartTime) -> Self {
		Self { start_time, ..self }
	}

	/// Sets the initial playback position of the instance (in seconds).
	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	/// Sets the volume of the instance.
	pub fn volume(self, volume: impl Into<Value>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the playback rate of the instance, as a factor of the
	/// normal playback rate.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub fn playback_rate(self, playback_rate: impl Into<Value>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	/// Sets the panning of the instance, where 0 is hard left
	/// and 1 is hard right.
	pub fn panning(self, panning: impl Into<Value>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	/// Sets whether the instance should play in reverse.
	pub fn reverse(self) -> Self {
		Self {
			reverse: true,
			..self
		}
	}

	/// Sets the looping behavior of the instance.
	pub fn loop_start(self, loop_start: impl Into<InstanceLoopStart>) -> Self {
		Self {
			loop_start: loop_start.into(),
			..self
		}
	}

	/// Sets the mixer track this instance should play on.
	pub fn track(self, track: impl Into<TrackId>) -> Self {
		Self {
			track: track.into(),
			..self
		}
	}

	/// Sets the tween used to fade in the instance from silence.
	pub fn fade_in_tween(self, fade_in_tween: impl Into<Option<Tween>>) -> Self {
		Self {
			fade_in_tween: fade_in_tween.into(),
			..self
		}
	}
}

impl Default for InstanceSettings {
	fn default() -> Self {
		Self::new()
	}
}
