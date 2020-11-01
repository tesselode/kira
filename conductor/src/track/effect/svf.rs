use std::f64::consts::PI;

use crate::stereo_sample::StereoSample;

use super::Effect;

// https://www.musicdsp.org/en/latest/Filters/142-state-variable-filter-chamberlin-version.html

fn drive(x: StereoSample, amount: f32) -> StereoSample {
	(x * amount).atan() / amount
}

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
	pub drive: f64,
}

impl Default for StateVariableFilterSettings {
	fn default() -> Self {
		Self {
			mode: StateVariableFilterMode::LowPass,
			cutoff: 1.0,
			resonance: 0.0,
			drive: 0.1,
		}
	}
}

pub struct StateVariableFilter {
	mode: StateVariableFilterMode,
	cutoff: f64,
	resonance: f64,
	drive: f64,
	low: StereoSample,
	band: StereoSample,
}

impl StateVariableFilter {
	pub fn new(settings: StateVariableFilterSettings) -> Self {
		Self {
			mode: settings.mode,
			cutoff: settings.cutoff,
			resonance: settings.resonance,
			drive: settings.drive,
			low: StereoSample::from_mono(0.0),
			band: StereoSample::from_mono(0.0),
		}
	}
}

impl Effect for StateVariableFilter {
	fn process(&mut self, _dt: f64, input: StereoSample) -> StereoSample {
		let cutoff = self.cutoff as f32;
		let drive_amount = self.drive as f32;
		self.low += self.band * cutoff;
		let high = input - self.low - self.band * ((1.0 - self.resonance) as f32);
		self.band += high * cutoff;
		self.low = (self.low * drive_amount).atan() / drive_amount;
		self.band = (self.band * drive_amount).atan() / drive_amount;
		match self.mode {
			StateVariableFilterMode::LowPass => self.low,
			StateVariableFilterMode::BandPass => self.band,
			StateVariableFilterMode::HighPass => drive(high, drive_amount),
			StateVariableFilterMode::Notch => drive(high + self.low, drive_amount),
		}
	}
}
