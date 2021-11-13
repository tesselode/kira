pub(crate) mod context;

use std::sync::{atomic::Ordering, Arc};

use ringbuf::Consumer;

use crate::{dsp::Frame, parameter::Parameter};

use self::context::Context;

use super::{command::Command, resources::Resources};

/// The playback state of a [`Renderer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererState {
	/// The [`Renderer`] is outputting audio.
	Playing,
	/// The [`Renderer`] is fading out, and when the
	/// fade-out is finished, the [`Renderer`] will
	/// pause processing.
	Pausing,
	/// The [`Renderer`] is paused and outputting silence.
	Paused,
}

impl RendererState {
	fn from_u8(state: u8) -> Self {
		match state {
			0 => Self::Playing,
			1 => Self::Pausing,
			2 => Self::Paused,
			_ => panic!("Not a valid RendererState"),
		}
	}
}

/// Produces [`Frame`]s of audio data to be consumed by a
/// low-level audio API.
///
/// You will probably not need to interact with [`Renderer`]s
/// directly unless you're writing a [`Backend`](super::Backend).
pub struct Renderer {
	context: Arc<Context>,
	resources: Resources,
	command_consumer: Consumer<Command>,
	state: RendererState,
	fade_volume: Parameter,
}

impl Renderer {
	pub(super) fn new(
		context: Arc<Context>,
		resources: Resources,
		command_consumer: Consumer<Command>,
	) -> Self {
		Self {
			context,
			resources,
			command_consumer,
			state: RendererState::Playing,
			fade_volume: Parameter::new(1.0),
		}
	}

	/// Called by the backend when it's time to process
	/// a new batch of samples.
	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
		self.resources.parameters.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Parameter(command) => self.resources.parameters.run_command(command),
				Command::Mixer(command) => self.resources.mixer.run_command(command),
				Command::Clock(command) => self.resources.clocks.run_command(command),
				Command::Pause(fade_out_tween) => {
					self.state = RendererState::Pausing;
					self.context
						.state
						.store(RendererState::Pausing as u8, Ordering::SeqCst);
					self.fade_volume.set(0.0, fade_out_tween);
				}
				Command::Resume(fade_in_tween) => {
					self.state = RendererState::Playing;
					self.context
						.state
						.store(RendererState::Playing as u8, Ordering::SeqCst);
					self.fade_volume.set(1.0, fade_in_tween);
				}
			}
		}
	}

	/// Produces the next [`Frame`] of audio.
	pub fn process(&mut self) -> Frame {
		if self
			.fade_volume
			.update(self.context.dt, &self.resources.clocks)
		{
			if self.state == RendererState::Pausing {
				self.state = RendererState::Paused;
			}
		}

		if self.state == RendererState::Paused {
			return Frame::ZERO;
		}
		if self.state == RendererState::Playing {
			self.resources
				.clocks
				.update(self.context.dt, &self.resources.parameters);
			self.resources
				.parameters
				.update(self.context.dt, &self.resources.clocks);
		}
		self.resources.sounds.process(
			self.context.dt,
			&self.resources.parameters,
			&self.resources.clocks,
			&mut self.resources.mixer,
		);
		let out = self
			.resources
			.mixer
			.process(self.context.dt, &self.resources.parameters);
		out * self.fade_volume.value() as f32
	}
}
