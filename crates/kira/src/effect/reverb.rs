//! Adds reverberations to a sound. Useful for simulating room tones.

mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use crate::{
	command::{read_commands_into_parameters, ValueChangeCommand},
	command_writers_and_readers,
	effect::Effect,
	frame::Frame,
	info::Info,
	tween::Parameter,
	Mix,
};
use all_pass::AllPassFilter;
use comb::CombFilter;

mod all_pass;
mod comb;

const NUM_COMB_FILTERS: usize = 8;
const NUM_ALL_PASS_FILTERS: usize = 4;
const GAIN: f32 = 0.015;
const STEREO_SPREAD: usize = 23;

#[derive(Debug)]
enum ReverbState {
	Uninitialized,
	Initialized {
		comb_filters: [(CombFilter, CombFilter); NUM_COMB_FILTERS],
		all_pass_filters: [(AllPassFilter, AllPassFilter); NUM_ALL_PASS_FILTERS],
	},
}

// This code is based on Freeverb by Jezar at Dreampoint, found here:
// http://blog.bjornroche.com/2012/06/freeverb-original-public-domain-code-by.html
struct Reverb {
	command_readers: CommandReaders,
	feedback: Parameter,
	damping: Parameter,
	stereo_width: Parameter,
	mix: Parameter<Mix>,
	state: ReverbState,
}

impl Reverb {
	/// Creates a new `Reverb` effect.
	#[must_use]
	fn new(settings: ReverbBuilder, command_readers: CommandReaders) -> Self {
		Self {
			command_readers,
			feedback: Parameter::new(settings.feedback, 0.9),
			damping: Parameter::new(settings.damping, 0.1),
			stereo_width: Parameter::new(settings.stereo_width, 1.0),
			mix: Parameter::new(settings.mix, Mix(0.5)),
			state: ReverbState::Uninitialized,
		}
	}

	fn init_filters(&mut self, sample_rate: u32) {
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
	}
}

impl Effect for Reverb {
	fn init(&mut self, sample_rate: u32) {
		self.init_filters(sample_rate);
	}

	fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.init_filters(sample_rate);
	}

	fn on_start_processing(&mut self) {
		read_commands_into_parameters!(self, feedback, damping, stereo_width, mix);
	}

	fn process(&mut self, input: &mut [Frame], dt: f64, info: &Info) {
		if let ReverbState::Initialized {
			comb_filters,
			all_pass_filters,
		} = &mut self.state
		{
			self.feedback.update(dt * input.len() as f64, info);
			self.damping.update(dt * input.len() as f64, info);
			self.stereo_width.update(dt * input.len() as f64, info);
			self.mix.update(dt * input.len() as f64, info);

			let feedback = self.feedback.value() as f32;
			let damping = self.damping.value() as f32;

			let num_frames = input.len();
			for (i, frame) in input.iter_mut().enumerate() {
				let time_in_chunk = (i + 1) as f64 / num_frames as f64;
				let stereo_width = self.stereo_width.interpolated_value(time_in_chunk) as f32;
				let mix = self.mix.interpolated_value(time_in_chunk).0;

				let mut output = Frame::ZERO;
				let mono_input = (frame.left + frame.right) * GAIN;
				// accumulate comb filters in parallel
				for comb_filter in comb_filters.iter_mut() {
					output.left += comb_filter.0.process(mono_input, feedback, damping);
					output.right += comb_filter.1.process(mono_input, feedback, damping);
				}
				// feed through all-pass filters in series
				for all_pass_filter in all_pass_filters.iter_mut() {
					output.left = all_pass_filter.0.process(output.left);
					output.right = all_pass_filter.1.process(output.right);
				}
				let wet_1 = stereo_width / 2.0 + 0.5;
				let wet_2 = (1.0 - stereo_width) / 2.0;
				let output = Frame::new(
					output.left * wet_1 + output.right * wet_2,
					output.right * wet_1 + output.left * wet_2,
				);
				*frame = output * mix.sqrt() + *frame * (1.0 - mix).sqrt()
			}
		} else {
			panic!("Reverb should be initialized before the first process call")
		}
	}
}

command_writers_and_readers! {
	set_feedback: ValueChangeCommand<f64>,
	set_damping: ValueChangeCommand<f64>,
	set_stereo_width: ValueChangeCommand<f64>,
	set_mix: ValueChangeCommand<Mix>,
}
