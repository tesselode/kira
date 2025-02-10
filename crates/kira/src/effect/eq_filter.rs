//! Adjusts the volume of a frequency range of a sound.

// Code is based on https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::f64::consts::PI;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	frame::Frame,
	info::Info,
	Decibels, Parameter,
};

use super::Effect;

const MIN_Q: f64 = 0.01;

struct EqFilter {
	command_readers: CommandReaders,
	kind: EqFilterKind,
	frequency: Parameter,
	gain: Parameter<Decibels>,
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
			gain: Parameter::new(builder.gain, Decibels::IDENTITY),
			q: Parameter::new(builder.q, 1.0),
			ic1eq: Frame::ZERO,
			ic2eq: Frame::ZERO,
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

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		self.frequency.update(dt * input.len() as f64, info);
		self.gain.update(dt * input.len() as f64, info);
		self.q.update(dt * input.len() as f64, info);

		let num_frames = input.len();
		for (i, frame) in input.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			let frequency = self.frequency.interpolated_value(time_in_chunk);
			let q = self.q.interpolated_value(time_in_chunk);
			let gain = self.gain.interpolated_value(time_in_chunk);

			let Coefficients {
				a1,
				a2,
				a3,
				m0,
				m1,
				m2,
			} = Coefficients::calculate(self.kind, frequency, q, gain, dt);
			let v3 = *frame - self.ic2eq;
			let v1 = self.ic1eq * (a1 as f32) + v3 * (a2 as f32);
			let v2 = self.ic2eq + self.ic1eq * (a2 as f32) + v3 * (a3 as f32);
			self.ic1eq = v1 * 2.0 - self.ic1eq;
			self.ic2eq = v2 * 2.0 - self.ic2eq;
			*frame = *frame * (m0 as f32) + v1 * (m1 as f32) + v2 * (m2 as f32)
		}
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

impl Coefficients {
	#[must_use]
	fn calculate(kind: EqFilterKind, frequency: f64, q: f64, gain: Decibels, dt: f64) -> Self {
		// In my testing, the filter goes unstable when the frequency exceeds half the sample rate,
		// so I'm clamping this value to 0.5
		let relative_frequency = (frequency * dt).clamp(0.0001, 0.5);
		let q = q.max(MIN_Q);
		match kind {
			EqFilterKind::Bell => {
				let a = 10.0f64.powf(gain.0 as f64 / 40.0);
				let g = (PI * relative_frequency).tan();
				let k = 1.0 / (q * a);
				let a1 = 1.0 / (1.0 + g * (g + k));
				let a2 = g * a1;
				let a3 = g * a2;
				let m0 = 1.0;
				let m1 = k * (a * a - 1.0);
				let m2 = 0.0;
				Self {
					a1,
					a2,
					a3,
					m0,
					m1,
					m2,
				}
			}
			EqFilterKind::LowShelf => {
				let a = 10.0f64.powf(gain.0 as f64 / 40.0);
				let g = (PI * relative_frequency).tan() / a.sqrt();
				let k = 1.0 / q;
				let a1 = 1.0 / (1.0 + g * (g + k));
				let a2 = g * a1;
				let a3 = g * a2;
				let m0 = 1.0;
				let m1 = k * (a - 1.0);
				let m2 = a * a - 1.0;
				Self {
					a1,
					a2,
					a3,
					m0,
					m1,
					m2,
				}
			}
			EqFilterKind::HighShelf => {
				let a = 10.0f64.powf(gain.0 as f64 / 40.0);
				let g = (PI * relative_frequency).tan() * a.sqrt();
				let k = 1.0 / q;
				let a1 = 1.0 / (1.0 + g * (g + k));
				let a2 = g * a1;
				let a3 = g * a2;
				let m0 = a * a;
				let m1 = k * (1.0 - a) * a;
				let m2 = 1.0 - a * a;
				Self {
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

command_writers_and_readers! {
	set_kind: EqFilterKind,
	set_frequency: ValueChangeCommand<f64>,
	set_gain: ValueChangeCommand<Decibels>,
	set_q: ValueChangeCommand<f64>,
}
