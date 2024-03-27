//! Reduces (or increases) the dynamic range of audio.

// Loosely based on https://www.musicdsp.org/en/latest/Effects/204-simple-compressor-class-c.html

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use ringbuf::HeapConsumer;

use std::time::Duration;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	tween::{Parameter, Tween, Value},
};

use super::Effect;

struct Compressor {
	command_consumer: HeapConsumer<Command>,
	threshold: Parameter,
	ratio: Parameter,
	attack_duration: Parameter<Duration>,
	release_duration: Parameter<Duration>,
	makeup_gain: Parameter,
	mix: Parameter,
	envelope_follower: [f32; 2],
}

impl Compressor {
	fn new(builder: CompressorBuilder, command_consumer: HeapConsumer<Command>) -> Self {
		Self {
			command_consumer,
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
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetThreshold(target, tween) => self.threshold.set(target, tween),
				Command::SetRatio(target, tween) => self.ratio.set(target, tween),
				Command::SetAttackDuration(target, tween) => {
					self.attack_duration.set(target, tween)
				}
				Command::SetReleaseDuration(target, tween) => {
					self.release_duration.set(target, tween)
				}
				Command::SetMakeupGain(target, tween) => self.makeup_gain.set(target, tween),
				Command::SetMix(target, tween) => self.mix.set(target, tween),
			}
		}
	}

	fn process(&mut self, input: Frame, dt: f64, clock_info_provider: &ClockInfoProvider) -> Frame {
		self.threshold.update(dt, clock_info_provider);
		self.ratio.update(dt, clock_info_provider);
		self.attack_duration.update(dt, clock_info_provider);
		self.release_duration.update(dt, clock_info_provider);
		self.makeup_gain.update(dt, clock_info_provider);
		self.mix.update(dt, clock_info_provider);

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
		let makeup_gain_linear = 10.0f32.powf(self.makeup_gain.value() as f32 / 20.0);
		let output = Frame {
			left: amplitude[0] * input.left,
			right: amplitude[1] * input.right,
		} * makeup_gain_linear;

		let mix = self.mix.value() as f32;
		output * mix.sqrt() + input * (1.0 - mix).sqrt()
	}
}

enum Command {
	SetThreshold(Value<f64>, Tween),
	SetRatio(Value<f64>, Tween),
	SetAttackDuration(Value<Duration>, Tween),
	SetReleaseDuration(Value<Duration>, Tween),
	SetMakeupGain(Value<f64>, Tween),
	SetMix(Value<f64>, Tween),
}
