use crate::{
	loop_behavior::LoopBehavior, parameter::Tween, start_time::StartTime,
	track::TrackId, value::Value,
};

/// The loop behavior for an instance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceLoopBehavior {
	/// Use the default loop behavior defined by the sound.
	DefaultForSound,
	/// Use a custom loop behavior.
	Custom(LoopBehavior),
	/// Do not loop.
	None,
}

impl InstanceLoopBehavior {
	pub(crate) fn as_option(self, default: Option<LoopBehavior>) -> Option<LoopBehavior> {
		match self {
			Self::DefaultForSound => default,
			Self::Custom(loop_behavior) => Some(loop_behavior),
			Self::None => None,
		}
	}
}

impl<T: Into<Option<LoopBehavior>>> From<T> for InstanceLoopBehavior {
	fn from(loop_behavior: T) -> Self {
		match loop_behavior.into() {
			Some(loop_behavior) => Self::Custom(loop_behavior),
			None => Self::None,
		}
	}
}

impl Default for InstanceLoopBehavior {
	fn default() -> Self {
		Self::DefaultForSound
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
	pub loop_behavior: InstanceLoopBehavior,
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
			loop_behavior: InstanceLoopBehavior::default(),
			track: TrackId::Main,
			fade_in_tween: None,
		}
	}

	/// Sets when the instance should start playing.
	pub fn start_time(self, start_time: impl Into<StartTime>) -> Self {
		Self {
			start_time: start_time.into(),
			..self
		}
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
	pub fn loop_behavior(self, loop_behavior: impl Into<InstanceLoopBehavior>) -> Self {
		Self {
			loop_behavior: loop_behavior.into(),
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
