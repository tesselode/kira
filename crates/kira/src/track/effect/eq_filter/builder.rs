use ringbuf::HeapRb;

use crate::{parameter::Value, track::effect::EffectBuilder};

use super::{EqFilter, EqFilterHandle, EqFilterKind};

const COMMAND_CAPACITY: usize = 8;

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
	type Handle = EqFilterHandle;

	fn build(self) -> (Box<dyn crate::track::effect::Effect>, Self::Handle) {
		let (command_producer, command_consumer) = HeapRb::new(COMMAND_CAPACITY).split();
		(
			Box::new(EqFilter::new(self, command_consumer)),
			EqFilterHandle { command_producer },
		)
	}
}
