use ringbuf::HeapRb;

use crate::{track::effect::EffectBuilder, tween::Value};

use super::{EqFilter, EqFilterHandle, EqFilterKind};

const COMMAND_CAPACITY: usize = 8;

pub struct EqFilterBuilder {
	pub kind: EqFilterKind,
	pub frequency: Value<f64>,
	pub gain: Value<f64>,
	pub q: Value<f64>,
}

impl EqFilterBuilder {
	pub fn new(
		kind: EqFilterKind,
		frequency: impl Into<Value<f64>>,
		gain: impl Into<Value<f64>>,
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

	fn build(self) -> (Box<dyn crate::track::effect::Effect>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		(
			Box::new(EqFilter::new(self, command_consumer)),
			EqFilterHandle { command_producer },
		)
	}
}
