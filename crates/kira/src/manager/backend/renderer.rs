use std::sync::{
	atomic::{AtomicU32, Ordering},
	Arc,
};

use crate::{
	clock::clock_info::ClockInfoProvider, dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
};

use super::resources::Resources;

pub(crate) struct RendererShared {
	pub(crate) sample_rate: AtomicU32,
}

impl RendererShared {
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
}

impl Renderer {
	pub(crate) fn new(sample_rate: u32, resources: Resources) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			shared: Arc::new(RendererShared::new(sample_rate)),
			resources,
		}
	}

	pub(crate) fn shared(&self) -> Arc<RendererShared> {
		self.shared.clone()
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
		self.resources.sounds.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();
		self.resources.spatial_scenes.on_start_processing();
		self.resources.modulators.on_start_processing();
	}

	/// Produces the next [`Frame`] of audio.
	pub fn process(&mut self) -> Frame {
		self.resources.modulators.process(
			self.dt,
			&ClockInfoProvider::new(&self.resources.clocks.0.resources),
		);
		self.resources.clocks.update(
			self.dt,
			&ModulatorValueProvider::new(&self.resources.modulators.0.resources),
		);
		self.resources.sounds.process(
			self.dt,
			&ClockInfoProvider::new(&self.resources.clocks.0.resources),
			&ModulatorValueProvider::new(&self.resources.modulators.0.resources),
			&mut self.resources.mixer,
			&mut self.resources.spatial_scenes,
		);
		self.resources.spatial_scenes.process(
			self.dt,
			&ClockInfoProvider::new(&self.resources.clocks.0.resources),
			&ModulatorValueProvider::new(&self.resources.modulators.0.resources),
			&mut self.resources.mixer,
		);
		self.resources.mixer.process(
			self.dt,
			&ClockInfoProvider::new(&self.resources.clocks.0.resources),
			&ModulatorValueProvider::new(&self.resources.modulators.0.resources),
		)
	}
}
