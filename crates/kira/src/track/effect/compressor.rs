// Loosely based on https://www.musicdsp.org/en/latest/Effects/204-simple-compressor-class-c.html

mod builder;

use std::time::Duration;

pub use builder::*;

use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider, parameter::Parameter,
};

use super::Effect;

struct Compressor {
	threshold: Parameter<f32>,
	ratio: Parameter<f32>,
	attack_duration: Parameter<Duration>,
	release_duration: Parameter<Duration>,
	envelope_follower: [f32; 2],
}

impl Compressor {
	fn new(builder: CompressorBuilder) -> Self {
		Self {
			threshold: Parameter::new(builder.threshold, CompressorBuilder::DEFAULT_THRESHOLD),
			ratio: Parameter::new(builder.ratio, CompressorBuilder::DEFAULT_RATIO),
			attack_duration: Parameter::new(
				builder.attack_duration,
				CompressorBuilder::DEFAULT_ATTACK_DURATION,
			),
			release_duration: Parameter::new(
				builder.release_duration,
				CompressorBuilder::DEFAULT_RELEASE_DURATION,
			),
			envelope_follower: [0.0; 2],
		}
	}
}

impl Effect for Compressor {
	fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.threshold
			.update(dt, clock_info_provider, modulator_value_provider);
		self.ratio
			.update(dt, clock_info_provider, modulator_value_provider);
		self.attack_duration
			.update(dt, clock_info_provider, modulator_value_provider);
		self.release_duration
			.update(dt, clock_info_provider, modulator_value_provider);

		let threshold = self.threshold.value();
		let ratio = self.ratio.value();
		let attack_duration = self.attack_duration.value();
		let release_duration = self.release_duration.value();

		let input_dbfs = [
			20.0 * input.left.abs().log10(),
			20.0 * input.right.abs().log10(),
		];
		let over_dbfs = input_dbfs.map(|input| (input - threshold).max(0.0));
		for (i, envelope_follower) in self.envelope_follower.iter_mut().enumerate() {
			let duration = if *envelope_follower > over_dbfs[i] {
				release_duration
			} else {
				attack_duration
			};
			let speed = (-1.0 / (duration.as_secs_f64() / dt)).exp() as f32;
			*envelope_follower = over_dbfs[i] + speed * (*envelope_follower - over_dbfs[i]);
		}
		let gain_reduction = self
			.envelope_follower
			.map(|envelope_follower| envelope_follower * ((1.0 / ratio) - 1.0));
		let amplitude = gain_reduction.map(|gain_reduction| 10.0f32.powf(gain_reduction / 20.0));
		Frame {
			left: amplitude[0] * input.left,
			right: amplitude[0] * input.right,
		}
	}
}
