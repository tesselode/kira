use crate::{
	sound::{transport::Transport, Sound},
	Frame, INTERNAL_BUFFER_SIZE,
};

use super::StaticSoundData;

pub struct StaticSound {
	data: StaticSoundData,
	transport: Transport,
}

impl StaticSound {
	pub fn new(data: StaticSoundData) -> Self {
		let transport = Transport::new(
			0,
			data.settings.loop_region,
			false,
			data.sample_rate,
			data.num_frames(),
		);
		Self { data, transport }
	}
}

impl Sound for StaticSound {
	fn sample_rate(&self) -> u32 {
		self.data.sample_rate
	}

	fn process(&mut self) -> [Frame; INTERNAL_BUFFER_SIZE] {
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		for frame in &mut frames {
			*frame = self
				.data
				.frame_at_index(self.transport.position)
				.unwrap_or_default();
			self.transport.increment_position(self.data.num_frames());
		}
		frames
	}

	fn finished(&self) -> bool {
		!self.transport.playing
	}
}
