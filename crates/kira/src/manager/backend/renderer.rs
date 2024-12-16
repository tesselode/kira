use std::sync::{
	atomic::{AtomicU32, Ordering},
	Arc,
};

use crate::Frame;

use super::resources::Resources;

#[derive(Debug)]
pub(crate) struct RendererShared {
	pub(crate) sample_rate: AtomicU32,
}

impl RendererShared {
	#[must_use]
	pub fn new(sample_rate: u32) -> Self {
		Self {
			sample_rate: AtomicU32::new(sample_rate),
		}
	}
}

/// Produces [`Frame`]s of audio data to be consumed by a
/// low-level audio API.
///
/// You will probably not need to interact with [`Renderer`]s
/// directly unless you're writing a [`Backend`](super::Backend).
pub struct Renderer {
	dt: f64,
	shared: Arc<RendererShared>,
	resources: Resources,
	internal_buffer_size: usize,
	temp_buffer: Vec<Frame>,
}

impl Renderer {
	#[must_use]
	pub(crate) fn new(
		shared: Arc<RendererShared>,
		internal_buffer_size: usize,
		resources: Resources,
	) -> Self {
		Self {
			dt: 1.0 / shared.sample_rate.load(Ordering::SeqCst) as f64,
			shared,
			resources,
			internal_buffer_size,
			temp_buffer: vec![Frame::ZERO; internal_buffer_size],
		}
	}

	/// Called by the backend when the sample rate of the
	/// audio output changes.
	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
		self.shared.sample_rate.store(sample_rate, Ordering::SeqCst);
		self.resources.mixer.on_change_sample_rate(sample_rate);
	}

	/// Called by the backend when it's time to process
	/// a new batch of samples.
	pub fn on_start_processing(&mut self) {
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();
		self.resources.listeners.on_start_processing();
		self.resources.modulators.on_start_processing();
	}

	/// Produces the next [`Frame`]s of audio.
	pub fn process(&mut self, out: &mut [f32], num_channels: u16) {
		for chunk in out.chunks_mut(self.internal_buffer_size * num_channels as usize) {
			self.process_chunk(chunk, num_channels);
		}
	}

	fn process_chunk(&mut self, chunk: &mut [f32], num_channels: u16) {
		let num_frames = chunk.len() / num_channels as usize;

		self.resources.modulators.process(
			self.dt * num_frames as f64,
			&self.resources.clocks,
			&self.resources.listeners,
		);
		self.resources.clocks.update(
			self.dt * num_frames as f64,
			&self.resources.modulators,
			&self.resources.listeners,
		);
		self.resources.listeners.update(
			self.dt * num_frames as f64,
			&self.resources.clocks,
			&self.resources.modulators,
		);

		self.resources.mixer.process(
			&mut self.temp_buffer[..num_frames],
			self.dt,
			&self.resources.clocks,
			&self.resources.modulators,
			&self.resources.listeners,
		);

		// convert from frames to requested number of channels
		for (i, channels) in chunk.chunks_mut(num_channels.into()).enumerate() {
			let frame = self.temp_buffer[i];
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
		self.temp_buffer.fill(Frame::ZERO);
	}
}
