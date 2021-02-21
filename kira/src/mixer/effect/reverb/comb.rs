use crate::Frame;

#[derive(Debug)]
pub struct CombFilter {
	filter_store: Frame,
	buffer: Vec<Frame>,
	current_index: usize,
}

impl CombFilter {
	pub fn new(buffer_size: usize) -> Self {
		Self {
			filter_store: Frame::from_mono(0.0),
			buffer: vec![Frame::from_mono(0.0); buffer_size],
			current_index: 0,
		}
	}

	pub fn process(&mut self, input: Frame, room_size: f32, damp: f32) -> Frame {
		let output = self.buffer[self.current_index];
		self.filter_store = output * (1.0 - damp) + self.filter_store * damp;
		self.buffer[self.current_index] = input + self.filter_store * room_size;
		self.current_index += 1;
		self.current_index %= self.buffer.len();
		output
	}
}
