#[derive(Debug, Copy, Clone)]
pub struct Mapping {
	pub input_range: (f64, f64),
	pub output_range: (f64, f64),
	pub clamp_bottom: bool,
	pub clamp_top: bool,
}

impl Default for Mapping {
	fn default() -> Self {
		Self {
			input_range: (0.0, 1.0),
			output_range: (0.0, 1.0),
			clamp_bottom: false,
			clamp_top: false,
		}
	}
}

impl Mapping {
	pub fn map(&self, input: f64) -> f64 {
		let relative_input =
			(input - self.input_range.0) / (self.input_range.1 - self.input_range.0);
		let mut output =
			self.output_range.0 + (self.output_range.1 - self.output_range.0) * relative_input;
		if self.clamp_bottom {
			output = output.max(self.output_range.0);
		}
		if self.clamp_top {
			output = output.min(self.output_range.1);
		}
		output
	}
}
