use crate::Frame;

use super::Effect;

#[derive(Debug)]
struct AllPassFilter {
	buffer: Vec<Frame>,
}

impl AllPassFilter {
	pub fn new(samples: usize) -> Self {
		Self {
			buffer: vec![Frame::from_mono(0.0); samples],
		}
	}

	pub fn process(&mut self, input: Frame, gain: f32) -> Frame {
		let output = self.buffer.pop().unwrap() + input * -gain;
		self.buffer.insert(0, input + output * gain);
		output
	}
}

#[derive(Debug)]
pub struct Reverb {
	filters: Vec<AllPassFilter>,
}

impl Reverb {
	pub fn new() -> Self {
		Self {
			filters: vec![
				AllPassFilter::new(97),
				AllPassFilter::new(151),
				AllPassFilter::new(251),
				AllPassFilter::new(349),
				AllPassFilter::new(547),
				AllPassFilter::new(653),
				AllPassFilter::new(853),
			],
		}
	}
}

impl Effect for Reverb {
	fn process(
		&mut self,
		dt: f64,
		input: Frame,
		parameters: &crate::parameter::Parameters,
	) -> Frame {
		let mut output = input;
		for filter in &mut self.filters {
			output = filter.process(output, 0.95);
		}
		output
	}
}
