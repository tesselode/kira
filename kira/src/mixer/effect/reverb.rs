use all_pass::AllPassFilter;
use comb::CombFilter;

use crate::Frame;

use super::Effect;

pub mod all_pass;
pub mod comb;

const NUM_COMB_FILTERS: usize = 8;
const NUM_ALL_PASS_FILTERS: usize = 4;

#[derive(Debug)]
pub struct Reverb {
	gain: f32,
	room_size: f32,
	damp: f32,
	wet: f32,
	wet_1: f32,
	wet_2: f32,
	dry: f32,
	width: f32,

	comb_filters: [CombFilter; NUM_COMB_FILTERS],
	all_pass_filters: [AllPassFilter; NUM_ALL_PASS_FILTERS],
}

impl Reverb {
	pub fn new() -> Self {
		let width = 1.0;
		let wet = 1.0 / 3.0;
		let wet_1 = wet * (width / 2.0 + 0.5);
		let wet_2 = wet * ((1.0 - width) / 2.0);
		Self {
			gain: 0.015,
			room_size: 0.9,
			damp: 0.1,
			wet,
			wet_1,
			wet_2,
			dry: 0.0,
			width,
			comb_filters: [
				CombFilter::new(1116),
				CombFilter::new(1188),
				CombFilter::new(1277),
				CombFilter::new(1356),
				CombFilter::new(1422),
				CombFilter::new(1491),
				CombFilter::new(1557),
				CombFilter::new(1617),
			],
			all_pass_filters: [
				AllPassFilter::new(556, 0.5),
				AllPassFilter::new(441, 0.5),
				AllPassFilter::new(341, 0.5),
				AllPassFilter::new(225, 0.5),
			],
		}
	}
}

impl Effect for Reverb {
	fn process(
		&mut self,
		dt: f64,
		mut input: crate::Frame,
		parameters: &crate::parameter::Parameters,
	) -> crate::Frame {
		let mut output = Frame::from_mono(0.0);
		input *= self.gain;
		for comb_filter in &mut self.comb_filters {
			output += comb_filter.process(input, self.room_size, self.damp);
		}
		for all_pass_filter in &mut self.all_pass_filters {
			output = all_pass_filter.process(output);
		}
		output
	}
}
