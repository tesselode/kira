use crate::{renderer::Renderer, Frame, INTERNAL_BUFFER_SIZE};

pub struct BufferedRenderer {
	renderer: Renderer,
	buffer: [Frame; INTERNAL_BUFFER_SIZE],
	current_frame_index: usize,
}

impl BufferedRenderer {
	pub fn new(renderer: Renderer) -> Self {
		Self {
			renderer,
			buffer: [Frame::ZERO; INTERNAL_BUFFER_SIZE],
			current_frame_index: 0,
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.renderer.on_change_sample_rate(sample_rate)
	}

	pub fn on_start_processing(&mut self) {
		self.renderer.on_start_processing()
	}

	pub fn process(&mut self, data: &mut [f32], num_channels: u16) {
		for channels in data.chunks_exact_mut(num_channels as usize) {
			if self.current_frame_index >= INTERNAL_BUFFER_SIZE {
				self.current_frame_index = 0;
				self.buffer = self.renderer.process();
			}
			let out = self.buffer[self.current_frame_index];
			if num_channels == 1 {
				channels[0] = (out.left + out.right) / 2.0;
			} else {
				channels[0] = out.left;
				channels[1] = out.right;
				/*
					if there's more channels, send silence to them. if we don't,
					we might get bad sounds outputted to those channels.
					(https://github.com/tesselode/kira/issues/50)
				*/
				for channel in channels.iter_mut().skip(2) {
					*channel = 0.0;
				}
			}
			self.current_frame_index += 1;
		}
	}
}
