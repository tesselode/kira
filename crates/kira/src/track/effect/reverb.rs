//! Adds reverberations to a sound.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use ringbuf::Consumer;

use crate::{
	clock::ClockTime,
	dsp::Frame,
	track::Effect,
	tween::{Tween, Tweener},
};
use all_pass::AllPassFilter;
use comb::CombFilter;

mod all_pass;
mod comb;

const NUM_COMB_FILTERS: usize = 8;
const NUM_ALL_PASS_FILTERS: usize = 4;
const GAIN: f32 = 0.015;
const STEREO_SPREAD: usize = 23;

enum Command {
	SetFeedback(f64, Tween),
	SetDamping(f64, Tween),
	SetStereoWidth(f64, Tween),
	SetMix(f64, Tween),
}

#[derive(Debug)]
enum ReverbState {
	Uninitialized,
	Initialized {
		comb_filters: [(CombFilter, CombFilter); NUM_COMB_FILTERS],
		all_pass_filters: [(AllPassFilter, AllPassFilter); NUM_ALL_PASS_FILTERS],
	},
}

/// A reverb effect. Useul for simulating room tones.
// This code is based on Freeverb by Jezar at Dreampoint, found here:
// http://blog.bjornroche.com/2012/06/freeverb-original-public-domain-code-by.html
pub struct Reverb {
	command_consumer: Consumer<Command>,
	feedback: Tweener,
	damping: Tweener,
	stereo_width: Tweener,
	mix: Tweener,
	state: ReverbState,
}

impl Reverb {
	/// Creates a new `Reverb` effect.
	fn new(settings: ReverbBuilder, command_consumer: Consumer<Command>) -> Self {
		Self {
			command_consumer,
			feedback: Tweener::new(settings.feedback),
			damping: Tweener::new(settings.damping),
			stereo_width: Tweener::new(settings.stereo_width),
			mix: Tweener::new(settings.mix),
			state: ReverbState::Uninitialized,
		}
	}
}

impl Effect for Reverb {
	fn init(&mut self, sample_rate: u32) {
		if let ReverbState::Uninitialized = &self.state {
			const REFERENCE_SAMPLE_RATE: u32 = 44100;

			let adjust_buffer_size = |buffer_size: usize| -> usize {
				let sample_rate_factor = (sample_rate as f64) / (REFERENCE_SAMPLE_RATE as f64);
				((buffer_size as f64) * sample_rate_factor) as usize
			};

			self.state = ReverbState::Initialized {
				comb_filters: [
					(
						CombFilter::new(adjust_buffer_size(1116)),
						CombFilter::new(adjust_buffer_size(1116 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1188)),
						CombFilter::new(adjust_buffer_size(1188 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1277)),
						CombFilter::new(adjust_buffer_size(1277 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1356)),
						CombFilter::new(adjust_buffer_size(1356 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1422)),
						CombFilter::new(adjust_buffer_size(1422 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1491)),
						CombFilter::new(adjust_buffer_size(1491 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1557)),
						CombFilter::new(adjust_buffer_size(1557 + STEREO_SPREAD)),
					),
					(
						CombFilter::new(adjust_buffer_size(1617)),
						CombFilter::new(adjust_buffer_size(1617 + STEREO_SPREAD)),
					),
				],
				all_pass_filters: [
					(
						AllPassFilter::new(adjust_buffer_size(556)),
						AllPassFilter::new(adjust_buffer_size(556 + STEREO_SPREAD)),
					),
					(
						AllPassFilter::new(adjust_buffer_size(441)),
						AllPassFilter::new(adjust_buffer_size(441 + STEREO_SPREAD)),
					),
					(
						AllPassFilter::new(adjust_buffer_size(341)),
						AllPassFilter::new(adjust_buffer_size(341 + STEREO_SPREAD)),
					),
					(
						AllPassFilter::new(adjust_buffer_size(225)),
						AllPassFilter::new(adjust_buffer_size(225 + STEREO_SPREAD)),
					),
				],
			}
		} else {
			panic!("Reverb should be in the uninitialized state before init");
		}
	}

	fn on_start_processing(&mut self) {
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::SetFeedback(feedback, tween) => self.feedback.set(feedback, tween),
				Command::SetDamping(damping, tween) => self.damping.set(damping, tween),
				Command::SetStereoWidth(stereo_width, tween) => {
					self.stereo_width.set(stereo_width, tween)
				}
				Command::SetMix(mix, tween) => self.mix.set(mix, tween),
			}
		}
	}

	fn process(&mut self, input: Frame, dt: f64) -> Frame {
		if let ReverbState::Initialized {
			comb_filters,
			all_pass_filters,
		} = &mut self.state
		{
			self.feedback.update(dt);
			self.damping.update(dt);
			self.stereo_width.update(dt);
			self.mix.update(dt);

			let feedback = self.feedback.value() as f32;
			let damping = self.damping.value() as f32;
			let stereo_width = self.stereo_width.value() as f32;

			let mut output = Frame::ZERO;
			let mono_input = (input.left + input.right) * GAIN;
			// accumulate comb filters in parallel
			for comb_filter in comb_filters {
				output.left += comb_filter.0.process(mono_input, feedback, damping);
				output.right += comb_filter.1.process(mono_input, feedback, damping);
			}
			// feed through all-pass filters in series
			for all_pass_filter in all_pass_filters {
				output.left = all_pass_filter.0.process(output.left);
				output.right = all_pass_filter.1.process(output.right);
			}
			let wet_1 = stereo_width / 2.0 + 0.5;
			let wet_2 = (1.0 - stereo_width) / 2.0;
			let output = Frame::new(
				output.left * wet_1 + output.right * wet_2,
				output.right * wet_1 + output.left * wet_2,
			);
			let mix = self.mix.value() as f32;
			output * mix.sqrt() + input * (1.0 - mix).sqrt()
		} else {
			panic!("Reverb should be initialized before the first process call")
		}
	}

	fn on_clock_tick(&mut self, time: ClockTime) {
		self.feedback.on_clock_tick(time);
		self.damping.on_clock_tick(time);
		self.stereo_width.on_clock_tick(time);
		self.mix.on_clock_tick(time);
	}
}
