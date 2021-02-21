use all_pass::AllPassFilter;
use comb::CombFilter;

use crate::Frame;

use super::Effect;

pub mod all_pass;
pub mod comb;

const NUM_COMB_FILTERS: usize = 8;
const NUM_ALL_PASS_FILTERS: usize = 4;
const STEREO_SPREAD: usize = 23;

#[derive(Debug)]
pub struct Reverb {
	gain: f32,
	room_size: f32,
	damp: f32,
	wet_1: f32,
	wet_2: f32,
	dry: f32,
	width: f32,

	comb_filters: [(CombFilter, CombFilter); NUM_COMB_FILTERS],
	all_pass_filters: [(AllPassFilter, AllPassFilter); NUM_ALL_PASS_FILTERS],
}

impl Reverb {
	pub fn new() -> Self {
		let width = 1.0;
		let wet_1 = width / 2.0 + 0.5;
		let wet_2 = (1.0 - width) / 2.0;
		Self {
			gain: 0.015,
			room_size: 0.9,
			damp: 0.1,
			wet_1,
			wet_2,
			dry: 0.0,
			width,
			comb_filters: [
				(CombFilter::new(1116), CombFilter::new(1116 + STEREO_SPREAD)),
				(CombFilter::new(1188), CombFilter::new(1188 + STEREO_SPREAD)),
				(CombFilter::new(1277), CombFilter::new(1277 + STEREO_SPREAD)),
				(CombFilter::new(1356), CombFilter::new(1356 + STEREO_SPREAD)),
				(CombFilter::new(1422), CombFilter::new(1422 + STEREO_SPREAD)),
				(CombFilter::new(1491), CombFilter::new(1491 + STEREO_SPREAD)),
				(CombFilter::new(1557), CombFilter::new(1557 + STEREO_SPREAD)),
				(CombFilter::new(1617), CombFilter::new(1617 + STEREO_SPREAD)),
			],
			all_pass_filters: [
				(
					AllPassFilter::new(556),
					AllPassFilter::new(556 + STEREO_SPREAD),
				),
				(
					AllPassFilter::new(441),
					AllPassFilter::new(441 + STEREO_SPREAD),
				),
				(
					AllPassFilter::new(341),
					AllPassFilter::new(341 + STEREO_SPREAD),
				),
				(
					AllPassFilter::new(225),
					AllPassFilter::new(225 + STEREO_SPREAD),
				),
			],
		}
	}
}

impl Effect for Reverb {
	fn process(
		&mut self,
		dt: f64,
		input: crate::Frame,
		parameters: &crate::parameter::Parameters,
	) -> crate::Frame {
		let mut output = Frame::from_mono(0.0);
		let input = (input.left + input.right) * self.gain;
		for comb_filter in &mut self.comb_filters {
			output.left += comb_filter.0.process(input, self.room_size, self.damp);
			output.right += comb_filter.1.process(input, self.room_size, self.damp);
		}
		for all_pass_filter in &mut self.all_pass_filters {
			output.left = all_pass_filter.0.process(output.left);
			output.right = all_pass_filter.1.process(output.right);
		}
		Frame::new(
			output.left * self.wet_1 + output.right * self.wet_2,
			output.right * self.wet_1 + output.left * self.wet_2,
		)
	}
}
