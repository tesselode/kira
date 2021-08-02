pub(crate) mod context;

use std::sync::{atomic::Ordering, Arc};

use ringbuf::Consumer;

use crate::{frame::Frame, parameter::Parameter};

use self::context::Context;

use super::{command::Command, resources::Resources};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererState {
	Playing,
	Pausing,
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

	pub fn on_start_processing(&mut self) {
		self.resources.sounds.on_start_processing();
		self.resources.instances.on_start_processing();
		self.resources.parameters.on_start_processing();
		self.resources.mixer.on_start_processing();
		self.resources.clocks.on_start_processing();

		while let Some(command) = self.command_consumer.pop() {
			match command {
				Command::Sound(command) => self.resources.sounds.run_command(command),
				Command::Instance(command) => self.resources.instances.run_command(command),
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
			return Frame::from_mono(0.0);
		}
		if self.state == RendererState::Playing {
			self.resources
				.clocks
				.update(self.context.dt, &self.resources.parameters);
			self.resources
				.parameters
				.update(self.context.dt, &self.resources.clocks);
		}
		self.resources.instances.process(
			self.context.dt,
			&self.resources.sounds,
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
