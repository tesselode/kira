/// A transformation from one range of values to another.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct Mapping {
	/// The input range of the mapping.
	pub input_range: (f64, f64),
	/// The corresponding output range of the mapping.
	pub output_range: (f64, f64),
	/// Whether values should be prevented from being
	/// less than the bottom of the output range.
	pub clamp_bottom: bool,
	/// Whether values should be prevented from being
	/// greater than the top of the output range.
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
	/// Transforms an input value to an output value using this mapping.
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
