//! Removes frequencies from a sound.

use std::f64::consts::PI;

use crate::{dsp::Frame, track::Effect};

// This filter code is based on the filter code from baseplug:
// https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs

/// The frequencies that the filter will remove.
#[derive(Debug, Copy, Clone)]
pub enum FilterMode {
	/// Removes frequencies above the cutoff frequency.
	LowPass,
	/// Removes frequencies above and below the cutoff frequency.
	BandPass,
	/// Removes frequencies below the cutoff frequency.
	HighPass,
	/// Removes frequencies around the cutoff frequency.
	Notch,
}

/// Settings for a [`Filter`].
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct FilterSettings {
	/// The frequencies that the filter will remove.
	pub mode: FilterMode,
	/// The cutoff frequency of the filter (in hertz).
	pub cutoff: f64,
	/// The resonance of the filter.
	///
	/// The resonance is a feedback effect that produces
	/// a distinctive "ringing" sound.
	pub resonance: f64,
	/// How much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means
	/// only the dry signal will be heard. `1.0` means
	/// only the wet signal will be heard.
	pub mix: f64,
}

impl FilterSettings {
	/// Creates a new `FilterSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the frequencies that the filter will remove.
	pub fn mode(self, mode: FilterMode) -> Self {
		Self { mode, ..self }
	}

	/// Sets the cutoff frequency of the filter (in hertz).
	pub fn cutoff(self, cutoff: f64) -> Self {
		Self { cutoff, ..self }
	}

	/// Sets the resonance of the filter.
	pub fn resonance(self, resonance: f64) -> Self {
		Self { resonance, ..self }
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn mix(self, mix: f64) -> Self {
		Self { mix, ..self }
	}
}

impl Default for FilterSettings {
	fn default() -> Self {
		Self {
			mode: FilterMode::LowPass,
			cutoff: 1000.0,
			resonance: 0.0,
			mix: 1.0,
		}
	}
}

/// An effect that removes frequencies from input audio.
pub struct Filter {
	mode: FilterMode,
	cutoff: f64,
	resonance: f64,
	mix: f64,
	ic1eq: Frame,
	ic2eq: Frame,
}

impl Filter {
	/// Creates a new filter.
	pub fn new(settings: FilterSettings) -> Self {
		Self {
			mode: settings.mode,
			cutoff: settings.cutoff,
			resonance: settings.resonance,
			mix: settings.mix,
			ic1eq: Frame::ZERO,
			ic2eq: Frame::ZERO,
		}
	}
}

impl Effect for Filter {
	fn process(&mut self, input: Frame, dt: f64) -> Frame {
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
		let output = match self.mode {
			FilterMode::LowPass => v2,
			FilterMode::BandPass => v1,
			FilterMode::HighPass => input - v1 * (k as f32) - v2,
			FilterMode::Notch => input - v1 * (k as f32),
		};
		let mix = self.mix as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}
