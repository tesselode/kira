//! Removes frequencies from a sound.

use std::f64::consts::PI;

use crate::{
	frame::Frame,
	parameter::Parameters,
	value::{CachedValue, Value},
};

use super::Effect;

// This filter code is based on the filter code from baseplug:
// https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs

/// The frequencies that the filter will remove.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize)
)]
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
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct FilterSettings {
	/// The frequencies that the filter will remove.
	pub mode: FilterMode,
	/// The cutoff frequency of the filter (in hertz).
	pub cutoff: Value<f64>,
	/// The resonance of the filter.
	///
	/// The resonance is a feedback effect that produces
	/// a distinctive "ringing" sound.
	pub resonance: Value<f64>,
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
	pub fn cutoff<V: Into<Value<f64>>>(self, cutoff: V) -> Self {
		Self {
			cutoff: cutoff.into(),
			..self
		}
	}

	/// Sets the resonance of the filter.
	pub fn resonance<V: Into<Value<f64>>>(self, resonance: V) -> Self {
		Self {
			resonance: resonance.into(),
			..self
		}
	}
}

impl Default for FilterSettings {
	fn default() -> Self {
		Self {
			mode: FilterMode::LowPass,
			cutoff: 10000.0.into(),
			resonance: 0.0.into(),
		}
	}
}

/// An effect that removes frequencies from input audio.
#[derive(Debug, Copy, Clone)]
pub struct Filter {
	mode: FilterMode,
	cutoff: CachedValue<f64>,
	resonance: CachedValue<f64>,
	ic1eq: Frame,
	ic2eq: Frame,
}

impl Filter {
	/// Creates a new filter.
	pub fn new(settings: FilterSettings) -> Self {
		Self {
			mode: settings.mode,
			cutoff: CachedValue::new(settings.cutoff, 10000.0).with_valid_range(20.0..20000.0),
			resonance: CachedValue::new(settings.resonance, 0.0).with_valid_range(0.0..1.0),
			ic1eq: Frame::from_mono(0.0),
			ic2eq: Frame::from_mono(0.0),
		}
	}
}

impl Effect for Filter {
	fn process(&mut self, dt: f64, input: Frame, parameters: &Parameters) -> Frame {
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
			FilterMode::LowPass => v2,
			FilterMode::BandPass => v1,
			FilterMode::HighPass => input - v1 * (k as f32) - v2,
			FilterMode::Notch => input - v1 * (k as f32),
		}
	}
}
