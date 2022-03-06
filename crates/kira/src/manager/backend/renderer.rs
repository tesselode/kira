use std::sync::{
	atomic::{AtomicU8, Ordering},
	Arc,
};

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	manager::{command::Command, MainPlaybackState},
	tween::Tweener,
	Volume,
};

use super::resources::Resources;

pub(crate) struct RendererShared {
	pub(super) state: AtomicU8,
}

impl RendererShared {
	pub fn new() -> Self {
		Self {
			state: AtomicU8::new(MainPlaybackState::Playing as u8),
		}
	}

	pub fn state(&self) -> MainPlaybackState {
		MainPlaybackState::from_u8(self.state.load(Ordering::SeqCst))
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
	command_consumer: HeapConsumer<Command>,
	state: MainPlaybackState,
	fade_volume: Tweener<Volume>,
}

impl Renderer {
	pub(crate) fn new(
		sample_rate: u32,
		resources: Resources,
		command_consumer: HeapConsumer<Command>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			shared: Arc::new(RendererShared::new()),
			resources,
			command_consumer,
			state: MainPlaybackState::Playing,
			fade_volume: Tweener::new(Volume::Decibels(0.0)),
		}
	}

	pub(crate) fn shared(&self) -> Arc<RendererShared> {
		self.shared.clone()
	}

	/// Called by the backend when the sample rate of the
	/// audio output changes.
	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.dt = 1.0 / sample_rate as f64;
		self.resources.mixer.on_change_sample_rate(sample_rate);
	}

	/// Called by the backend when it's time to process
	/// a new batch of samples.
	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();
		self.resources.spatial_scenes.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Mixer(command) => self.resources.mixer.run_command(command),
				Command::Clock(command) => self.resources.clocks.run_command(command),
				Command::SpatialScene(command) => {
					self.resources.spatial_scenes.run_command(command)
				}
				Command::Pause(fade_out_tween) => {
					self.state = MainPlaybackState::Pausing;
					self.shared
						.state
						.store(MainPlaybackState::Pausing as u8, Ordering::SeqCst);
					self.fade_volume
						.set(Volume::Decibels(Volume::MIN_DECIBELS), fade_out_tween);
				}
				Command::Resume(fade_in_tween) => {
					self.state = MainPlaybackState::Playing;
					self.shared
						.state
						.store(MainPlaybackState::Playing as u8, Ordering::SeqCst);
					self.fade_volume.set(Volume::Decibels(0.0), fade_in_tween);
				}
			}
		}
	}

	/// Produces the next [`Frame`] of audio.
	pub fn process(&mut self) -> Frame {
		if self
			.fade_volume
			.update(self.dt, &ClockInfoProvider::new(&self.resources.clocks))
		{
			if self.state == MainPlaybackState::Pausing {
				self.state = MainPlaybackState::Paused;
			}
		}
		if self.state == MainPlaybackState::Paused {
			return Frame::ZERO;
		}
		if self.state == MainPlaybackState::Playing {
			self.resources.clocks.update(self.dt);
		}
		self.resources.sounds.process(
			self.dt,
			&ClockInfoProvider::new(&self.resources.clocks),
			&mut self.resources.mixer,
			&mut self.resources.spatial_scenes,
		);
		self.resources
			.spatial_scenes
			.process(&mut self.resources.mixer);
		let out = self
			.resources
			.mixer
			.process(self.dt, &ClockInfoProvider::new(&self.resources.clocks));
		out * self.fade_volume.value().as_amplitude() as f32
	}
}
