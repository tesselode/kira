//! Removes frequencies from a sound.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;
use ringbuf::Consumer;

use std::f64::consts::PI;

use crate::{
	clock::ClockTime,
	dsp::Frame,
	track::Effect,
	tween::{Tween, Tweenable},
};

enum Command {
	SetMode(FilterMode),
	SetCutoff(f64, Tween),
	SetResonance(f64, Tween),
	SetMix(f64, Tween),
}

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

/// An effect that removes frequencies from input audio.
pub struct Filter {
	command_consumer: Consumer<Command>,
	mode: FilterMode,
	cutoff: Tweenable,
	resonance: Tweenable,
	mix: Tweenable,
	ic1eq: Frame,
	ic2eq: Frame,
}

impl Filter {
	/// Creates a new filter.
	fn new(builder: FilterBuilder, command_consumer: Consumer<Command>) -> Self {
		Self {
			command_consumer,
			mode: builder.mode,
			cutoff: Tweenable::new(builder.cutoff),
			resonance: Tweenable::new(builder.resonance),
			mix: Tweenable::new(builder.mix),
			ic1eq: Frame::ZERO,
			ic2eq: Frame::ZERO,
		}
	}
}

impl Effect for Filter {
	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetMode(mode) => self.mode = mode,
				Command::SetCutoff(cutoff, tween) => self.cutoff.set(cutoff, tween),
				Command::SetResonance(resonance, tween) => self.resonance.set(resonance, tween),
				Command::SetMix(mix, tween) => self.mix.set(mix, tween),
			}
		}
	}

	fn process(&mut self, input: Frame, dt: f64) -> Frame {
		self.cutoff.update(dt);
		self.resonance.update(dt);
		self.mix.update(dt);
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

	fn on_clock_tick(&mut self, time: ClockTime) {
		self.cutoff.on_clock_tick(time);
		self.resonance.on_clock_tick(time);
		self.mix.on_clock_tick(time);
	}
}
