//! Removes frequencies from a sound.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::f64::consts::PI;

use crate::{
	clock::clock_info::ClockInfoProvider, command::ValueChangeCommand, command_writers_and_readers,
	dsp::Frame, modulator::value_provider::ModulatorValueProvider, track::Effect, tween::Parameter,
};

// This filter code is based on the filter code from baseplug:
// https://github.com/wrl/baseplug/blob/trunk/examples/svf/svf_simper.rs

/// The frequencies that the filter will remove.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

struct Filter {
	command_readers: CommandReaders,
	mode: FilterMode,
	cutoff: Parameter,
	resonance: Parameter,
	mix: Parameter,
	ic1eq: Frame,
	ic2eq: Frame,
}

impl Filter {
	/// Creates a new filter.
	fn new(builder: FilterBuilder, command_readers: CommandReaders) -> Self {
		Self {
			mode: builder.mode,
			cutoff: Parameter::new(builder.cutoff, 1000.0),
			resonance: Parameter::new(builder.resonance, 0.0),
			mix: Parameter::new(builder.mix, 1.0),
			ic1eq: Frame::ZERO,
			ic2eq: Frame::ZERO,
			command_readers,
		}
	}
}

impl Effect for Filter {
	fn on_start_processing(&mut self) {
		if let Some(mode) = self.command_readers.mode_change.read().copied() {
			self.mode = mode;
		}
		self.cutoff
			.read_commands(&mut self.command_readers.cutoff_change);
		self.resonance
			.read_commands(&mut self.command_readers.resonance_change);
		self.mix.read_commands(&mut self.command_readers.mix_change);
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.cutoff
			.update(dt, clock_info_provider, modulator_value_provider);
		self.resonance
			.update(dt, clock_info_provider, modulator_value_provider);
		self.mix
			.update(dt, clock_info_provider, modulator_value_provider);
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
		let output = match self.mode {
			FilterMode::LowPass => v2,
			FilterMode::BandPass => v1,
			FilterMode::HighPass => input - v1 * (k as f32) - v2,
			FilterMode::Notch => input - v1 * (k as f32),
		};
		let mix = self.mix.value() as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}

command_writers_and_readers!(
	mode_change: FilterMode,
	cutoff_change: ValueChangeCommand<f64>,
	resonance_change: ValueChangeCommand<f64>,
	mix_change: ValueChangeCommand<f64>
);
