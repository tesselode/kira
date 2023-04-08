use crate::{parameter::Value, track::effect::EffectBuilder};

use super::{EqFilter, EqFilterKind};

pub struct EqFilterBuilder {
	pub kind: EqFilterKind,
	pub frequency: Value<f32>,
	pub gain: Value<f32>,
	pub q: Value<f32>,
}

impl EqFilterBuilder {
	pub fn new(
		kind: EqFilterKind,
		frequency: impl Into<Value<f32>>,
		gain: impl Into<Value<f32>>,
		q: impl Into<Value<f32>>,
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
	type Handle = ();

	fn build(self) -> (Box<dyn crate::track::effect::Effect>, Self::Handle) {
		(Box::new(EqFilter::new(self)), ())
	}
}
