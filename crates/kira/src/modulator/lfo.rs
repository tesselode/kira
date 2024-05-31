//! Oscillates back and forth.

#[cfg(test)]
mod test;

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::{
	f64::consts::TAU,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

use crate::{
	clock::clock_info::ClockInfoProvider, command::read_commands_into_parameters,
	command::ValueChangeCommand, command_writers_and_readers, tween::Parameter,
};

use super::{value_provider::ModulatorValueProvider, Modulator};

struct Lfo {
	waveform: Waveform,
	frequency: Parameter,
	amplitude: Parameter,
	offset: Parameter,
	command_readers: CommandReaders,
	shared: Arc<LfoShared>,
	phase: f64,
	value: f64,
}

impl Lfo {
	#[must_use]
	fn new(builder: &LfoBuilder, command_readers: CommandReaders, shared: Arc<LfoShared>) -> Self {
		Self {
			waveform: builder.waveform,
			frequency: Parameter::new(builder.frequency, 2.0),
			amplitude: Parameter::new(builder.amplitude, 1.0),
			offset: Parameter::new(builder.offset, 0.0),
			command_readers,
			shared,
			phase: builder.starting_phase / TAU,
			value: 0.0,
		}
	}
}

impl Modulator for Lfo {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, frequency, amplitude, offset);
		if let Some(waveform) = self.command_readers.set_waveform.read() {
			self.waveform = waveform;
		}
		if let Some(phase) = self.command_readers.set_phase.read() {
			self.phase = phase / TAU;
		}
	}

	fn update(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) {
		self.frequency
			.update(dt, clock_info_provider, modulator_value_provider);
		self.amplitude
			.update(dt, clock_info_provider, modulator_value_provider);
		self.offset
			.update(dt, clock_info_provider, modulator_value_provider);
		self.phase += dt * self.frequency.value();
		self.phase %= 1.0;
		self.value = self.offset.value() + self.amplitude.value() * self.waveform.value(self.phase);
	}

	fn value(&self) -> f64 {
		self.value
	}

	fn finished(&self) -> bool {
		self.shared.removed.load(Ordering::SeqCst)
	}
}

/// Describes an oscillation pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Waveform {
	/// The value moves back and forth smoothly.
	Sine,
	/// The value moves back and forth at a constant speed.
	Triangle,
	/// The value moves gradually in one direction, then abruptly jumps in the other.
	Saw,
	/// The value jumps back and forth between two values.
	Pulse {
		/// The ratio between how much time the oscillator spends on one value vs. the other.
		///
		/// This should be a number between `0.0` and `1.0`. A value of `0.5` means the oscillator
		/// spends an equal amount of time at both values.
		width: f64,
	},
}

impl Waveform {
	#[must_use]
	fn value(self, phase: f64) -> f64 {
		match self {
			Waveform::Sine => (phase * TAU).sin(),
			Waveform::Triangle => ((phase + 0.75).fract() - 0.5).abs() * 4.0 - 1.0,
			Waveform::Saw => (phase + 0.5).fract() * 2.0 - 1.0,
			Waveform::Pulse { width } => {
				if phase < width {
					1.0
				} else {
					-1.0
				}
			}
		}
	}
}

#[derive(Debug)]
struct LfoShared {
	removed: AtomicBool,
}

impl LfoShared {
	fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}
}

command_writers_and_readers! {
	set_waveform: Waveform,
	set_frequency: ValueChangeCommand<f64>,
	set_amplitude: ValueChangeCommand<f64>,
	set_offset: ValueChangeCommand<f64>,
	set_phase: f64,
}
