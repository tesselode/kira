#[derive(Debug)]
pub struct CombFilter {
	filter_store: f32,
	buffer: Vec<f32>,
	current_index: usize,
}

impl CombFilter {
	pub fn new(buffer_size: usize) -> Self {
		Self {
			filter_store: 0.0,
			buffer: vec![0.0; buffer_size],
			current_index: 0,
		}
	}

	pub fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
		let output = self.buffer[self.current_index];
		self.filter_store = output * (1.0 - damp) + self.filter_store * damp;
		self.buffer[self.current_index] = input + self.filter_store * feedback;
		self.current_index += 1;
		self.current_index %= self.buffer.len();
		output
	}
}
