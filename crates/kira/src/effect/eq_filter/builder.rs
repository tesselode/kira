use crate::{Decibels, Value, effect::EffectBuilder};

use super::{EqFilter, EqFilterHandle, EqFilterKind, command_writers_and_readers};

/// Configures an EQ filter.
pub struct EqFilterBuilder {
	/// The shape of the frequency adjustment curve.
	pub kind: EqFilterKind,
	/// The "center" or "corner" of the frequency range to adjust in Hz
	/// (for bell or shelf curves, respectively).
	pub frequency: Value<f64>,
	/// The volume adjustment for frequencies in the specified range (in decibels).
	pub gain: Value<Decibels>,
	/// The width of the frequency range to adjust.
	///
	/// A higher Q value results in a narrower range of frequencies being adjusted.
	/// The value should be greater than `0.0`.
	pub q: Value<f64>,
}

impl EqFilterBuilder {
	/// Creates a new `EqFilterBuilder`.
	#[must_use]
	pub fn new(
		kind: EqFilterKind,
		frequency: impl Into<Value<f64>>,
		gain: impl Into<Value<Decibels>>,
		q: impl Into<Value<f64>>,
	) -> Self {
		Self {
			kind,
			frequency: frequency.into(),
			gain: gain.into(),
			q: q.into(),
		}
	}
}

impl EffectBuilder for EqFilterBuilder {
	type Handle = EqFilterHandle;

	fn build(self) -> (Box<dyn crate::effect::Effect>, Self::Handle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		(
			Box::new(EqFilter::new(self, command_readers)),
			EqFilterHandle { command_writers },
		)
	}
}
