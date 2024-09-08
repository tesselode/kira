//! Reduces (or increases) the dynamic range of audio.

// Loosely based on https://www.musicdsp.org/en/latest/Effects/204-simple-compressor-class-c.html

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::time::Duration;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	frame::Frame,
	info::Info,
	tween::Parameter,
	Dbfs, Mix,
};

use super::Effect;

struct Compressor {
	command_readers: CommandReaders,
	threshold: Parameter,
	ratio: Parameter,
	attack_duration: Parameter<Duration>,
	release_duration: Parameter<Duration>,
	makeup_gain: Parameter<Dbfs>,
	mix: Parameter<Mix>,
	envelope_follower: [f32; 2],
}

impl Compressor {
	#[must_use]
	fn new(builder: CompressorBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
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
			makeup_gain: Parameter::new(
				builder.makeup_gain,
				CompressorBuilder::DEFAULT_MAKEUP_GAIN,
			),
			mix: Parameter::new(builder.mix, CompressorBuilder::DEFAULT_MIX),
			envelope_follower: [0.0; 2],
		}
	}
}

impl Effect for Compressor {
	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(
			self,
			threshold,
			ratio,
			attack_duration,
			release_duration,
			makeup_gain,
			mix,
		);
	}

	fn process(&mut self, input: Frame, dt: f64, info: &Info) -> Frame {
		self.threshold.update(dt, info);
		self.ratio.update(dt, info);
		self.attack_duration.update(dt, info);
		self.release_duration.update(dt, info);
		self.makeup_gain.update(dt, info);
		self.mix.update(dt, info);

		let threshold = self.threshold.value() as f32;
		let ratio = self.ratio.value() as f32;
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
			let speed = (-1.0 / (duration.as_secs_f64() / dt)).exp();
			*envelope_follower = over_dbfs[i] + speed as f32 * (*envelope_follower - over_dbfs[i]);
		}
		let gain_reduction = self
			.envelope_follower
			.map(|envelope_follower| envelope_follower * ((1.0 / ratio) - 1.0));
		let amplitude = gain_reduction.map(|gain_reduction| 10.0f32.powf(gain_reduction / 20.0));
		let makeup_gain_linear = 10.0f32.powf(self.makeup_gain.value().0 / 20.0);
		let output = Frame {
			left: amplitude[0] * input.left,
			right: amplitude[1] * input.right,
		} * makeup_gain_linear;

		let mix = self.mix.value().0 as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}

command_writers_and_readers! {
	set_threshold: ValueChangeCommand<f64>,
	set_ratio: ValueChangeCommand<f64>,
	set_attack_duration: ValueChangeCommand<Duration>,
	set_release_duration: ValueChangeCommand<Duration>,
	set_makeup_gain: ValueChangeCommand<Dbfs>,
	set_mix: ValueChangeCommand<Mix>,
}
