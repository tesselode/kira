use crate::{
	sound::{IntoOptionalRegion, PlaybackPosition, PlaybackRate, Region},
	tween::{Tween, Value},
	Dbfs, StartTime,
};

/// Settings for a streaming sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamingSoundSettings {
	/// When the sound should start playing.
	pub start_time: StartTime,
	/// Where in the sound playback should start.
	pub start_position: PlaybackPosition,
	/// The portion of the sound that should be looped.
	pub loop_region: Option<Region>,
	/// The volume of the sound.
	pub volume: Value<Dbfs>,
	/// The playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub playback_rate: Value<PlaybackRate>,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value<f64>,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}

impl StreamingSoundSettings {
	/// Creates a new [`StreamingSoundSettings`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			start_time: StartTime::Immediate,
			start_position: PlaybackPosition::Seconds(0.0),
			loop_region: None,
			volume: Value::Fixed(Dbfs::MAX),
			playback_rate: Value::Fixed(PlaybackRate::Factor(1.0)),
			panning: Value::Fixed(0.5),
			fade_in_tween: None,
		}
	}

	/** Sets when the sound should start playing. */
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
	pub fn start_time(self, start_time: impl Into<StartTime>) -> Self {
		Self {
			start_time: start_time.into(),
			..self
		}
	}

	/// Sets where in the sound playback should start.
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
	pub fn start_position(self, start_position: impl Into<PlaybackPosition>) -> Self {
		Self {
			start_position: start_position.into(),
			..self
		}
	}

	/** Sets the portion of the sound that should be looped. */
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
	pub fn loop_region(self, loop_region: impl IntoOptionalRegion) -> Self {
		Self {
			loop_region: loop_region.into_optional_region(),
			..self
		}
	}

	/** Sets the volume of the sound. */
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
	pub fn volume(self, volume: impl Into<Value<Dbfs>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/**
	Sets the playback rate of the sound.

	Changing the playback rate will change both the speed
	and the pitch of the sound.
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
	pub fn playback_rate(self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	/**
	Sets the panning of the sound, where 0 is hard left
	and 1 is hard right.
	*/
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
	pub fn panning(self, panning: impl Into<Value<f64>>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	/// Sets the tween used to fade in the instance from silence.
	#[must_use = "This method consumes self and returns a modified StreamingSoundSettings, so the return value should be used"]
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
