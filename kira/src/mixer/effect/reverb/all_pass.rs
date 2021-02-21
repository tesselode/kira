use crate::Frame;

#[derive(Debug)]
pub struct AllPassFilter {
	feedback: f32,
	buffer: Vec<Frame>,
	current_index: usize,
}

impl AllPassFilter {
	pub fn new(buffer_size: usize, feedback: f32) -> Self {
		Self {
			feedback,
			buffer: vec![Frame::from_mono(0.0); buffer_size],
			current_index: 0,
		}
	}

	pub fn process(&mut self, input: Frame) -> Frame {
		let buffer_output = self.buffer[self.current_index];
		let output = -input + buffer_output;
		self.buffer[self.current_index] = input + buffer_output * self.feedback;
		self.current_index += 1;
		self.current_index %= self.buffer.len();
		output
	}
}
