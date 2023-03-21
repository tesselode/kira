#[cfg(test)]
mod test;

mod builder;
mod handle;

use std::{
	f64::consts::TAU,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};

pub use builder::*;
use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	parameter::{Parameter, Value},
	tween::Tween,
};

use super::{value_provider::ModulatorValueProvider, Modulator};

struct Lfo {
	waveform: Waveform,
	frequency: Parameter,
	amplitude: Parameter,
	offset: Parameter,
	command_consumer: HeapConsumer<Command>,
	shared: Arc<LfoShared>,
	phase: f64,
	value: f64,
}

impl Lfo {
	fn new(
		builder: &LfoBuilder,
		command_consumer: HeapConsumer<Command>,
		shared: Arc<LfoShared>,
	) -> Self {
		Self {
			waveform: builder.waveform,
			frequency: Parameter::new(builder.frequency, 2.0),
			amplitude: Parameter::new(builder.amplitude, 1.0),
			offset: Parameter::new(builder.offset, 0.0),
			command_consumer,
			shared,
			phase: 0.0,
			value: 0.0,
		}
	}
}

impl Modulator for Lfo {
	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetFrequency { target, tween } => self.frequency.set(target, tween),
				Command::SetAmplitude { target, tween } => self.amplitude.set(target, tween),
				Command::SetOffset { target, tween } => self.offset.set(target, tween),
			}
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
	Sine,
	Triangle,
	Saw,
	Pulse { width: f64 },
}

impl Waveform {
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

enum Command {
	SetFrequency { target: Value<f64>, tween: Tween },
	SetAmplitude { target: Value<f64>, tween: Tween },
	SetOffset { target: Value<f64>, tween: Tween },
}

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
