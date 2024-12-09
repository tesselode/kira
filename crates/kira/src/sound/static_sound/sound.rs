use crate::{
	sound::{transport::Transport, Sound},
	Frame,
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

	fn process(&mut self) -> Frame {
		let frame = self
			.data
			.frame_at_index(self.transport.position)
			.unwrap_or_default();
		self.transport.increment_position(self.data.num_frames());
		frame
	}

	fn finished(&self) -> bool {
		!self.transport.playing
	}
}
