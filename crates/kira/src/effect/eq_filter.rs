//! Adjusts the volume of a frequency range of a sound.

// Code is based on https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::f64::consts::PI;

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	frame::Frame,
	listener::ListenerInfoProvider,
	modulator::value_provider::ModulatorValueProvider,
	tween::Parameter,
};

use super::Effect;

const MIN_Q: f64 = 0.01;

struct EqFilter {
	command_readers: CommandReaders,
	kind: EqFilterKind,
	frequency: Parameter,
	gain: Parameter,
	q: Parameter,
	ic1eq: Frame,
	ic2eq: Frame,
}

impl EqFilter {
	#[must_use]
	fn new(builder: EqFilterBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			kind: builder.kind,
			frequency: Parameter::new(builder.frequency, 500.0),
			gain: Parameter::new(builder.gain, 0.0),
			q: Parameter::new(builder.q, 1.0),
			ic1eq: Frame::ZERO,
			ic2eq: Frame::ZERO,
		}
	}

	#[must_use]
	fn calculate_coefficients(&self, dt: f64) -> Coefficients {
		// In my testing, the filter goes unstable when the frequency exceeds half the sample rate,
		// so I'm clamping this value to 0.5
		let relative_frequency = (self.frequency.value() * dt).clamp(0.0, 0.5);
		let q = self.q.value().max(MIN_Q);
		match self.kind {
			EqFilterKind::Bell => {
				let a = 10.0f64.powf(self.gain.value() / 40.0);
				let g = (PI * relative_frequency).tan();
				let k = 1.0 / (q * a);
				let a1 = 1.0 / (1.0 + g * (g + k));
				let a2 = g * a1;
				let a3 = g * a2;
				let m0 = 1.0;
				let m1 = k * (a * a - 1.0);
				let m2 = 0.0;
				Coefficients {
					a1,
					a2,
					a3,
					m0,
					m1,
					m2,
				}
			}
			EqFilterKind::LowShelf => {
				let a = 10.0f64.powf(self.gain.value() / 40.0);
				let g = (PI * relative_frequency).tan() / a.sqrt();
				let k = 1.0 / q;
				let a1 = 1.0 / (1.0 + g * (g + k));
				let a2 = g * a1;
				let a3 = g * a2;
				let m0 = 1.0;
				let m1 = k * (a - 1.0);
				let m2 = a * a - 1.0;
				Coefficients {
					a1,
					a2,
					a3,
					m0,
					m1,
					m2,
				}
			}
			EqFilterKind::HighShelf => {
				let a = 10.0f64.powf(self.gain.value() / 40.0);
				let g = (PI * relative_frequency).tan() * a.sqrt();
				let k = 1.0 / q;
				let a1 = 1.0 / (1.0 + g * (g + k));
				let a2 = g * a1;
				let a3 = g * a2;
				let m0 = a * a;
				let m1 = k * (1.0 - a) * a;
				let m2 = 1.0 - a * a;
				Coefficients {
					a1,
					a2,
					a3,
					m0,
					m1,
					m2,
				}
			}
		}
	}
}

impl Effect for EqFilter {
	fn on_start_processing(&mut self) {
		if let Some(kind) = self.command_readers.set_kind.read() {
			self.kind = kind;
		}
		read_commands_into_parameters!(self, frequency, gain, q);
	}

	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		listener_info_provider: &ListenerInfoProvider,
	) -> Frame {
		self.frequency.update(
			dt,
			clock_info_provider,
			modulator_value_provider,
			listener_info_provider,
		);
		self.gain.update(
			dt,
			clock_info_provider,
			modulator_value_provider,
			listener_info_provider,
		);
		self.q.update(
			dt,
			clock_info_provider,
			modulator_value_provider,
			listener_info_provider,
		);
		let Coefficients {
			a1,
			a2,
			a3,
			m0,
			m1,
			m2,
		} = self.calculate_coefficients(dt);
		let v3 = input - self.ic2eq;
		let v1 = self.ic1eq * (a1 as f32) + v3 * (a2 as f32);
		let v2 = self.ic2eq + self.ic1eq * (a2 as f32) + v3 * (a3 as f32);
		self.ic1eq = v1 * 2.0 - self.ic1eq;
		self.ic2eq = v2 * 2.0 - self.ic2eq;
		input * (m0 as f32) + v1 * (m1 as f32) + v2 * (m2 as f32)
	}
}

/// The shape of the frequency adjustment curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EqFilterKind {
	/// Frequencies around the user-defined frequency are adjusted.
	Bell,
	/// Frequencies around and lower than the user-defined frequency are adjusted.
	LowShelf,
	/// Frequencies around and higher than the user-defined frequency are adjusted.
	HighShelf,
}

struct Coefficients {
	a1: f64,
	a2: f64,
	a3: f64,
	m0: f64,
	m1: f64,
	m2: f64,
}

command_writers_and_readers! {
	set_kind: EqFilterKind,
	set_frequency: ValueChangeCommand<f64>,
	set_gain: ValueChangeCommand<f64>,
	set_q: ValueChangeCommand<f64>,
}
