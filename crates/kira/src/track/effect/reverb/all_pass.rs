const FEEDBACK: f32 = 0.5;

#[derive(Debug)]
pub struct AllPassFilter {
	buffer: Vec<f32>,
	current_index: usize,
}

impl AllPassFilter {
	pub fn new(buffer_size: usize) -> Self {
		Self {
			buffer: vec![0.0; buffer_size],
			current_index: 0,
		}
	}

	pub fn process(&mut self, input: f32) -> f32 {
		let buffer_output = self.buffer[self.current_index];
		let output = -input + buffer_output;
		self.buffer[self.current_index] = input + buffer_output * FEEDBACK;
		self.current_index += 1;
		self.current_index %= self.buffer.len();
		output
	}
}
