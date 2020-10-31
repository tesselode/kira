use std::f64::consts::PI;

use crate::stereo_sample::StereoSample;

use super::Effect;

// https://www.musicdsp.org/en/latest/Filters/142-state-variable-filter-chamberlin-version.html

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
	low: StereoSample,
	band: StereoSample,
}

impl StateVariableFilter {
	pub fn new(settings: StateVariableFilterSettings) -> Self {
		Self {
			mode: settings.mode,
			cutoff: settings.cutoff,
			resonance: settings.resonance,
			low: StereoSample::from_mono(0.0),
			band: StereoSample::from_mono(0.0),
		}
	}
}

impl Effect for StateVariableFilter {
	fn process(&mut self, _dt: f64, input: StereoSample) -> StereoSample {
		self.low += self.band * (self.cutoff as f32);
		let high = input - self.low - self.band * ((1.0 - self.resonance) as f32);
		self.band += high * (self.cutoff as f32);
		match self.mode {
			StateVariableFilterMode::LowPass => self.low,
			StateVariableFilterMode::BandPass => self.band,
			StateVariableFilterMode::HighPass => high,
			StateVariableFilterMode::Notch => high + self.low,
		}
	}
}
