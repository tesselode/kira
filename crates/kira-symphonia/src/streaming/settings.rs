use kira::{track::TrackId, tween::Tween, value::Value, LoopBehavior, StartTime};

/// Settings for a streaming sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamingSoundSettings {
	/// When the instance should start playing.
	pub start_time: StartTime,
	/// The initial playback position of the sound (in seconds).
	pub start_position: f64,
	/// The volume of the sound.
	pub volume: Value,
	/// The playback rate of the sound, as a factor of the
	/// normal playback rate.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub playback_rate: Value,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value,
	/// The looping behavior of the sound.
	pub loop_behavior: Option<LoopBehavior>,
	/// The mixer track this sound should play on.
	pub track: TrackId,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}

impl StreamingSoundSettings {
	/// Creates a new [`StreamingSoundSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			start_time: StartTime::Immediate,
			start_position: 0.0,
			volume: Value::Fixed(1.0),
			playback_rate: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			loop_behavior: None,
			track: TrackId::Main,
			fade_in_tween: None,
		}
	}

	/// Sets when the sound should start playing.
	pub fn start_time(self, start_time: impl Into<StartTime>) -> Self {
		Self {
			start_time: start_time.into(),
			..self
		}
	}

	/// Sets the initial playback position of the sound (in seconds).
	pub fn start_position(self, start_position: f64) -> Self {
		Self {
			start_position,
			..self
		}
	}

	/// Sets the volume of the sound.
	pub fn volume(self, volume: impl Into<Value>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the playback rate of the sound, as a factor of the
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

	/// Sets the panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub fn panning(self, panning: impl Into<Value>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	/// Sets the looping behavior of the sound.
	pub fn loop_behavior(self, loop_behavior: impl Into<Option<LoopBehavior>>) -> Self {
		Self {
			loop_behavior: loop_behavior.into(),
			..self
		}
	}

	/// Sets the mixer track this sound should play on.
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

impl Default for StreamingSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
