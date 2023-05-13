use crate::{
	sound::{IntoOptionalRegion, Region},
	tween::{Tween, Value},
	OutputDestination, PlaybackRate, StartTime, Volume,
};

/// Settings for a static sound.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct StaticSoundSettings {
	/// When the sound should start playing.
	pub start_time: StartTime,
	/// The portion of the sound that should be played.
	pub playback_region: Region,
	/// The portion of the sound that should be looped.
	pub loop_region: Option<Region>,
	/// Whether the sound should be played in reverse.
	pub reverse: bool,
	/// The volume of the sound.
	pub volume: Value<Volume>,
	/// The playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub playback_rate: Value<PlaybackRate>,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value<f64>,
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
			playback_region: Region::default(),
			reverse: false,
			loop_region: None,
			volume: Value::Fixed(Volume::Amplitude(1.0)),
			playback_rate: Value::Fixed(PlaybackRate::Factor(1.0)),
			panning: Value::Fixed(0.5),
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

	/// Sets the portion of the sound that should be played.
	pub fn playback_region(self, playback_region: impl Into<Region>) -> Self {
		Self {
			playback_region: playback_region.into(),
			..self
		}
	}

	/// Sets whether the sound should be played in reverse.
	pub fn reverse(self, reverse: bool) -> Self {
		Self { reverse, ..self }
	}

	/// Sets the portion of the sound that should be looped.
	pub fn loop_region(self, loop_region: impl IntoOptionalRegion) -> Self {
		Self {
			loop_region: loop_region.into_optional_loop_region(),
			..self
		}
	}

	/// Sets the volume of the sound.
	pub fn volume(self, volume: impl Into<Value<Volume>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub fn playback_rate(self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	/// Sets the panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub fn panning(self, panning: impl Into<Value<f64>>) -> Self {
		Self {
			panning: panning.into(),
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
