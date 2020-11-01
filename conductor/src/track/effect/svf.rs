use std::f64::consts::PI;

use crate::stereo_sample::StereoSample;

use super::Effect;

// https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs

pub enum StateVariableFilterMode {
	LowPass,
	BandPass,
	HighPass,
	Notch,
}

pub struct StateVariableFilterSettings {
	pub mode: StateVariableFilterMode,
	pub cutoff: f64,
	pub resonance: f64,
}

impl Default for StateVariableFilterSettings {
	fn default() -> Self {
		Self {
			mode: StateVariableFilterMode::LowPass,
			cutoff: 1.0,
			resonance: 0.0,
		}
	}
}

pub struct StateVariableFilter {
	mode: StateVariableFilterMode,
	cutoff: f64,
	resonance: f64,
	ic1eq: StereoSample,
	ic2eq: StereoSample,
}

impl StateVariableFilter {
	pub fn new(settings: StateVariableFilterSettings) -> Self {
		Self {
			mode: settings.mode,
			cutoff: settings.cutoff,
			resonance: settings.resonance,
			ic1eq: StereoSample::from_mono(0.0),
			ic2eq: StereoSample::from_mono(0.0),
		}
	}
}

impl Effect for StateVariableFilter {
	fn process(&mut self, dt: f64, input: StereoSample) -> StereoSample {
		let sample_rate = 1.0 / dt;
		let g = (PI * (self.cutoff / sample_rate)).tan();
		let k = 2.0 - (1.9 * self.resonance.min(1.0).max(0.0));
		let a1 = 1.0 / (1.0 + (g * (g + k)));
		let a2 = g * a1;
		let a3 = g * a2;
		let v3 = input - self.ic2eq;
		let v1 = (self.ic1eq * (a1 as f32)) + (v3 * (a2 as f32));
		let v2 = self.ic2eq + (self.ic1eq * (a2 as f32)) + (v3 * (a3 as f32));
		self.ic1eq = (v1 * 2.0) - self.ic1eq;
		self.ic2eq = (v2 * 2.0) - self.ic2eq;
		v2
	}
}
