pub(crate) mod context;

use std::sync::{atomic::Ordering, Arc};

use ringbuf::Consumer;

use crate::{
	dsp::Frame,
	manager::{command::Command, MainPlaybackState},
	tween::Tweener,
	Volume,
};

use self::context::Context;

use super::resources::Resources;

/// Produces [`Frame`]s of audio data to be consumed by a
/// low-level audio API.
///
/// You will probably not need to interact with [`Renderer`]s
/// directly unless you're writing a [`Backend`](super::Backend).
pub struct Renderer {
	context: Arc<Context>,
	resources: Resources,
	command_consumer: Consumer<Command>,
	state: MainPlaybackState,
	fade_volume: Tweener<Volume>,
}

impl Renderer {
	pub(crate) fn new(
		context: Arc<Context>,
		resources: Resources,
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			context,
			resources,
			command_consumer,
			state: MainPlaybackState::Playing,
			fade_volume: Tweener::new(Volume::Decibels(0.0)),
		}
	}

	/// Called by the backend when it's time to process
	/// a new batch of samples.
	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Mixer(command) => self.resources.mixer.run_command(command),
				Command::Clock(command) => self.resources.clocks.run_command(command),
				Command::Pause(fade_out_tween) => {
					self.state = MainPlaybackState::Pausing;
					self.context
						.state
						.store(MainPlaybackState::Pausing as u8, Ordering::SeqCst);
					self.fade_volume
						.set(Volume::Decibels(Volume::MIN_DECIBELS), fade_out_tween);
				}
				Command::Resume(fade_in_tween) => {
					self.state = MainPlaybackState::Playing;
					self.context
						.state
						.store(MainPlaybackState::Playing as u8, Ordering::SeqCst);
					self.fade_volume.set(Volume::Decibels(0.0), fade_in_tween);
				}
			}
		}
	}

	/// Produces the next [`Frame`] of audio.
	pub fn process(&mut self) -> Frame {
		if self.fade_volume.update(self.context.dt) {
			if self.state == MainPlaybackState::Pausing {
				self.state = MainPlaybackState::Paused;
			}
		}

		if self.state == MainPlaybackState::Paused {
			return Frame::ZERO;
		}
		if self.state == MainPlaybackState::Playing {
			let clock_tick_events = self.resources.clocks.update(self.context.dt);
			for time in clock_tick_events {
				self.resources.sounds.on_clock_tick(*time);
				self.resources.mixer.on_clock_tick(*time);
			}
		}
		self.resources
			.sounds
			.process(self.context.dt, &mut self.resources.mixer);
		let out = self.resources.mixer.process(self.context.dt);
		out * self.fade_volume.value().as_amplitude() as f32
	}
}
