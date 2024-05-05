use crate::{
	effect::{Effect, EffectBuilder},
	tween::Value,
};

use super::{command_writers_and_readers, Reverb, ReverbHandle};

/// Configures a reverb effect.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ReverbBuilder {
	/// How much the room reverberates. A higher value will
	/// result in a bigger sounding room. 1.0 gives an infinitely
	/// reverberating room.
	pub feedback: Value<f64>,
	/// How quickly high frequencies disappear from the reverberation.
	pub damping: Value<f64>,
	/// The stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	pub stereo_width: Value<f64>,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: Value<f64>,
}

impl ReverbBuilder {
	/// Creates a new [`ReverbBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets how much the room reverberates. A higher value will
	/// result in a bigger sounding room. 1.0 gives an infinitely
	/// reverberating room.
	#[must_use = "This method consumes self and returns a modified ReverbBuilder, so the return value should be used"]
	pub fn feedback(self, feedback: impl Into<Value<f64>>) -> Self {
		Self {
			feedback: feedback.into(),
			..self
		}
	}

	/// Sets how quickly high frequencies disappear from the reverberation.
	#[must_use = "This method consumes self and returns a modified ReverbBuilder, so the return value should be used"]
	pub fn damping(self, damping: impl Into<Value<f64>>) -> Self {
		Self {
			damping: damping.into(),
			..self
		}
	}

	/// Sets the stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	#[must_use = "This method consumes self and returns a modified ReverbBuilder, so the return value should be used"]
	pub fn stereo_width(self, stereo_width: impl Into<Value<f64>>) -> Self {
		Self {
			stereo_width: stereo_width.into(),
			..self
		}
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	#[must_use = "This method consumes self and returns a modified ReverbBuilder, so the return value should be used"]
	pub fn mix(self, mix: impl Into<Value<f64>>) -> Self {
		Self {
			mix: mix.into(),
			..self
		}
	}
}

impl Default for ReverbBuilder {
	fn default() -> Self {
		Self {
			feedback: Value::Fixed(0.9),
			damping: Value::Fixed(0.1),
			stereo_width: Value::Fixed(1.0),
			mix: Value::Fixed(0.5),
		}
	}
}

impl EffectBuilder for ReverbBuilder {
	type Handle = ReverbHandle;

	fn build(self) -> (Box<dyn Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(Reverb::new(self, command_readers)),
			ReverbHandle { command_writers },
		)
	}
}
