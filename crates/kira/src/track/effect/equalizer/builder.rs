use crate::{parameter::Value, track::effect::EffectBuilder};

use super::{BandKind, Equalizer};

pub struct EqualizerBuilder {
	pub bands: Vec<BandSettings>,
}

impl EqualizerBuilder {
	pub fn new() -> Self {
		Self { bands: vec![] }
	}

	pub fn add_band(
		&mut self,
		kind: BandKind,
		frequency: impl Into<Value<f32>>,
		gain: impl Into<Value<f32>>,
		q: impl Into<Value<f32>>,
	) {
		self.bands.push(BandSettings {
			kind,
			frequency: frequency.into(),
			gain: gain.into(),
			q: q.into(),
		});
	}
}

impl Default for EqualizerBuilder {
	fn default() -> Self {
		Self::new()
	}
}

impl EffectBuilder for EqualizerBuilder {
	type Handle = ();

	fn build(self) -> (Box<dyn crate::track::effect::Effect>, Self::Handle) {
		(Box::new(Equalizer::new(self)), ())
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandSettings {
	pub kind: BandKind,
	pub frequency: Value<f32>,
	pub gain: Value<f32>,
	pub q: Value<f32>,
}
