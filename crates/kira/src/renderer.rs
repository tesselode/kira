use crate::{resources::Resources, Frame, INTERNAL_BUFFER_SIZE};

pub struct Renderer {
	dt: f64,
	resources: Resources,
}

impl Renderer {
	pub(crate) fn new(sample_rate: u32, resources: Resources) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			resources,
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
	}

	pub fn on_start_processing(&mut self) {
		self.resources.clocks.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.modulators.on_start_processing();
	}

	pub fn process(&mut self, out: &mut [f32], num_channels: u16) {
		for chunk in out.chunks_mut(INTERNAL_BUFFER_SIZE * num_channels as usize) {
			self.process_chunk(chunk, num_channels);
		}
	}

	fn process_chunk(&mut self, chunk: &mut [f32], num_channels: u16) {
		let num_frames = chunk.len() / num_channels as usize;

		// update modulation sources (one frame at a time so they can react
		// to each other sample-accurately)
		self.resources.clocks.reset_buffers();
		self.resources.modulators.reset_buffers();
		for _ in 0..num_frames {
			self.resources
				.clocks
				.update(self.dt, &self.resources.modulators.0.resources);
			self.resources
				.modulators
				.update(self.dt, &self.resources.clocks.0.resources);
		}

		// process sounds in chunks
		let mut frames = [Frame::ZERO; INTERNAL_BUFFER_SIZE];
		self.resources.mixer.process(
			&mut frames[..num_frames],
			self.dt,
			&self.resources.clocks,
			&self.resources.modulators,
		);

		// convert from frames to requested number of channels
		for (i, channels) in chunk.chunks_mut(num_channels.into()).enumerate() {
			let frame = frames[i];
			if num_channels == 1 {
				channels[0] = (frame.left + frame.right) / 2.0;
			} else {
				channels[0] = frame.left;
				channels[1] = frame.right;
				/*
					if there's more channels, send silence to them. if we don't,
					we might get bad sounds outputted to those channels.
					(https://github.com/tesselode/kira/issues/50)
				*/
				for channel in channels.iter_mut().skip(2) {
					*channel = 0.0;
				}
			}
		}
	}
}
