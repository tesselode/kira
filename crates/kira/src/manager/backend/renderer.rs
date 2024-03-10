use std::sync::{
	atomic::{AtomicU32, AtomicU8, Ordering},
	Arc,
};

use ringbuf::HeapConsumer;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	manager::{command::Command, MainPlaybackState},
	modulator::value_provider::ModulatorValueProvider,
	tween::{Parameter, Value},
	Volume,
};

use super::resources::Resources;

pub(crate) struct RendererShared {
	pub(crate) state: AtomicU8,
	pub(crate) sample_rate: AtomicU32,
}

impl RendererShared {
	pub fn new(sample_rate: u32) -> Self {
		Self {
			state: AtomicU8::new(MainPlaybackState::Playing as u8),
			sample_rate: AtomicU32::new(sample_rate),
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
	fade_volume: Parameter<Volume>,
}

impl Renderer {
	pub(crate) fn new(
		sample_rate: u32,
		resources: Resources,
		command_consumer: HeapConsumer<Command>,
	) -> Self {
		Self {
			dt: 1.0 / sample_rate as f64,
			shared: Arc::new(RendererShared::new(sample_rate)),
			resources,
			command_consumer,
			state: MainPlaybackState::Playing,
			fade_volume: Parameter::new(Value::Fixed(Volume::Decibels(0.0)), Volume::Decibels(0.0)),
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
		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::AddSound(key, sound_wrapper) => {
					self.resources.sounds.add(key, sound_wrapper)
				}
				Command::AddSubTrack(id, sub_track) => {
					self.resources.mixer.add_sub_track(id, sub_track)
				}
				Command::SpatialScene(command) => {
					self.resources.spatial_scenes.run_command(command)
				}
				Command::Modulator(command) => self.resources.modulators.run_command(command),
				Command::Pause(fade_out_tween) => {
					self.state = MainPlaybackState::Pausing;
					self.shared
						.state
						.store(MainPlaybackState::Pausing as u8, Ordering::SeqCst);
					self.fade_volume.set(
						Value::Fixed(Volume::Decibels(Volume::MIN_DECIBELS)),
						fade_out_tween,
					);
				}
				Command::Resume(fade_in_tween) => {
					self.state = MainPlaybackState::Playing;
					self.shared
						.state
						.store(MainPlaybackState::Playing as u8, Ordering::SeqCst);
					self.fade_volume
						.set(Value::Fixed(Volume::Decibels(0.0)), fade_in_tween);
				}
			}
		}

		self.resources.sounds.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();
		self.resources.spatial_scenes.on_start_processing();
		self.resources.modulators.on_start_processing();
	}

	/// Produces the next [`Frame`] of audio.
	pub fn process(&mut self) -> Frame {
		if self.fade_volume.update(
			self.dt,
			&ClockInfoProvider::new(self.resources.clocks.items()),
			&ModulatorValueProvider::new(&self.resources.modulators.modulators),
		) {
			if self.state == MainPlaybackState::Pausing {
				self.state = MainPlaybackState::Paused;
			}
		}
		if self.state == MainPlaybackState::Paused {
			return Frame::ZERO;
		}
		if self.state == MainPlaybackState::Playing {
			self.resources.modulators.process(
				self.dt,
				&ClockInfoProvider::new(self.resources.clocks.items()),
			);
			self.resources.clocks.for_each(|clock, other_clocks| {
				clock.update(
					self.dt,
					&ClockInfoProvider::new(other_clocks),
					&ModulatorValueProvider::new(&self.resources.modulators.modulators),
				);
			});
		}
		self.resources.sounds.process(
			self.dt,
			&ClockInfoProvider::new(self.resources.clocks.items()),
			&ModulatorValueProvider::new(&self.resources.modulators.modulators),
			&mut self.resources.mixer,
			&mut self.resources.spatial_scenes,
		);
		self.resources.spatial_scenes.process(
			self.dt,
			&ClockInfoProvider::new(self.resources.clocks.items()),
			&ModulatorValueProvider::new(&self.resources.modulators.modulators),
			&mut self.resources.mixer,
		);
		let out = self.resources.mixer.process(
			self.dt,
			&ClockInfoProvider::new(self.resources.clocks.items()),
			&ModulatorValueProvider::new(&self.resources.modulators.modulators),
		);
		out * self.fade_volume.value().as_amplitude() as f32
	}
}
