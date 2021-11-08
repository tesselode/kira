use kira::{value::Value, LoopBehavior};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamingSoundSettings {
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
}

impl StreamingSoundSettings {
	pub fn new() -> Self {
		Self {
			start_position: 0.0,
			volume: Value::Fixed(1.0),
			playback_rate: Value::Fixed(1.0),
			panning: Value::Fixed(0.5),
			loop_behavior: None,
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
}

impl Default for StreamingSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
