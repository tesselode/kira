mod builder;

pub use builder::*;

use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider, parameter::Parameter,
};

use super::Effect;

struct Compressor {
	threshold: Parameter<f32>,
	ratio: Parameter<f32>,
	attack_speed: Parameter<f32>,
	release_speed: Parameter<f32>,
	relative_gain_reduction: [f32; 2],
}

impl Compressor {
	fn new(builder: CompressorBuilder) -> Self {
		Self {
			threshold: Parameter::new(builder.threshold, -12.0),
			ratio: Parameter::new(builder.ratio, 2.0),
			attack_speed: Parameter::new(builder.attack_speed, 1.0),
			release_speed: Parameter::new(builder.release_speed, 1.0),
			relative_gain_reduction: [0.0; 2],
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
		self.attack_speed
			.update(dt, clock_info_provider, modulator_value_provider);
		self.release_speed
			.update(dt, clock_info_provider, modulator_value_provider);

		let threshold = self.threshold.value();
		let ratio = self.ratio.value();
		let attack_speed = self.attack_speed.value();
		let release_speed = self.release_speed.value();

		let input_dbfs = [
			20.0 * input.left.abs().log10(),
			20.0 * input.right.abs().log10(),
		];
		let target_gain_reduction = [
			if input_dbfs[0] >= threshold { 1.0 } else { 0.0 },
			if input_dbfs[1] >= threshold { 1.0 } else { 0.0 },
		];
		let mut output_dbfs = [0.0; 2];
		for i in 0..2 {
			let speed = if self.relative_gain_reduction[i] > target_gain_reduction[i] {
				release_speed
			} else {
				attack_speed
			};
			self.relative_gain_reduction[i] +=
				(target_gain_reduction[i] - self.relative_gain_reduction[i]) * speed * dt as f32;
			let current_ratio = 1.0 + (ratio - 1.0) * self.relative_gain_reduction[i];
			output_dbfs[i] =
				threshold.min(input_dbfs[i]) + (input_dbfs[i] - threshold).max(0.0) / current_ratio;
		}
		Frame {
			left: 10.0f32.powf(output_dbfs[0] / 20.0) * input.left.signum(),
			right: 10.0f32.powf(output_dbfs[1] / 20.0) * input.right.signum(),
		}
	}
}
