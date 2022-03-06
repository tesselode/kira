use crate::{tween::Tween, LoopBehavior, OutputDestination, PlaybackRate, StartTime, Volume};

/// Settings for a static sound.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct StaticSoundSettings {
	/// When the sound should start playing.
	pub start_time: StartTime,
	/// The initial playback position of the sound (in seconds).
	pub start_position: f64,
	/// The volume of the sound.
	pub volume: Volume,
	/// The playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub playback_rate: PlaybackRate,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: f64,
	/// Whether the sound should play in reverse.
	///
	/// If set to `true`, the start position will be relative
	/// to the end of the sound.
	pub reverse: bool,
	/// The looping behavior of the sound.
	pub loop_behavior: Option<LoopBehavior>,
	/// The destination that this sound should be routed to.
	pub output_destination: OutputDestination,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}

impl StaticSoundSettings {
	/// Creates a new [`StaticSoundSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			start_time: StartTime::default(),
			start_position: 0.0,
			volume: Volume::Amplitude(1.0),
			playback_rate: PlaybackRate::Factor(1.0),
			panning: 0.5,
			reverse: false,
			loop_behavior: None,
			output_destination: OutputDestination::default(),
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
	pub fn volume(self, volume: impl Into<Volume>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub fn playback_rate(self, playback_rate: impl Into<PlaybackRate>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	/// Sets the panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub fn panning(self, panning: f64) -> Self {
		Self { panning, ..self }
	}

	/// Sets whether the sound should play in reverse.
	pub fn reverse(self, reverse: bool) -> Self {
		Self { reverse, ..self }
	}

	/// Sets the looping behavior of the sound.
	pub fn loop_behavior(self, loop_behavior: impl Into<Option<LoopBehavior>>) -> Self {
		Self {
			loop_behavior: loop_behavior.into(),
			..self
		}
	}

	/// Sets the destination that this sound should be routed to.
	pub fn output_destination(self, output_destination: impl Into<OutputDestination>) -> Self {
		Self {
			output_destination: output_destination.into(),
			..self
		}
	}

	/// Sets the tween used to fade in the sound from silence.
	pub fn fade_in_tween(self, fade_in_tween: impl Into<Option<Tween>>) -> Self {
		Self {
			fade_in_tween: fade_in_tween.into(),
			..self
		}
	}
}

impl Default for StaticSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
