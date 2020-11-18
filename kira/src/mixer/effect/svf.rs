use std::f64::consts::PI;

use crate::{
	parameter::Parameters,
	stereo_sample::StereoSample,
	value::{CachedValue, Value},
};

use super::Effect;

// https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs

#[derive(Debug, Copy, Clone)]
pub enum StateVariableFilterMode {
	LowPass,
	BandPass,
	HighPass,
	Notch,
}

#[derive(Debug, Clone)]
pub struct StateVariableFilterSettings {
	pub mode: StateVariableFilterMode,
	pub cutoff: Value<f64>,
	pub resonance: Value<f64>,
}

impl Default for StateVariableFilterSettings {
	fn default() -> Self {
		Self {
			mode: StateVariableFilterMode::LowPass,
			cutoff: 1.0.into(),
			resonance: 0.0.into(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct StateVariableFilter {
	mode: StateVariableFilterMode,
	cutoff: CachedValue<f64>,
	resonance: CachedValue<f64>,
	ic1eq: StereoSample,
	ic2eq: StereoSample,
}

impl StateVariableFilter {
	pub fn new(settings: StateVariableFilterSettings) -> Self {
		Self {
			mode: settings.mode,
			cutoff: CachedValue::new(settings.cutoff, 10000.0),
			resonance: CachedValue::new(settings.resonance, 0.0),
			ic1eq: StereoSample::from_mono(0.0),
			ic2eq: StereoSample::from_mono(0.0),
		}
	}
}

impl Effect for StateVariableFilter {
	fn process(&mut self, dt: f64, input: StereoSample, parameters: &Parameters) -> StereoSample {
		self.cutoff.update(parameters);
		self.resonance.update(parameters);
		let sample_rate = 1.0 / dt;
		let g = (PI * (self.cutoff.value() / sample_rate)).tan();
		let k = 2.0 - (1.9 * self.resonance.value().min(1.0).max(0.0));
		let a1 = 1.0 / (1.0 + (g * (g + k)));
		let a2 = g * a1;
		let a3 = g * a2;
		let v3 = input - self.ic2eq;
		let v1 = (self.ic1eq * (a1 as f32)) + (v3 * (a2 as f32));
		let v2 = self.ic2eq + (self.ic1eq * (a2 as f32)) + (v3 * (a3 as f32));
		self.ic1eq = (v1 * 2.0) - self.ic1eq;
		self.ic2eq = (v2 * 2.0) - self.ic2eq;
		match self.mode {
			StateVariableFilterMode::LowPass => v2,
			StateVariableFilterMode::BandPass => v1,
			StateVariableFilterMode::HighPass => input - v1 * (k as f32) - v2,
			StateVariableFilterMode::Notch => input - v1 * (k as f32),
		}
	}
}
