mod builder;
mod handle;

use std::sync::Arc;

pub use builder::*;
pub use handle::*;

use crate::{
	arena::Key,
	command::{CommandReader, ValueChangeCommand},
	effect::Effect,
	info::Info,
	resources::{clocks::Clocks, modulators::Modulators},
	tween::Parameter,
	Decibels, Frame, INTERNAL_BUFFER_SIZE,
};

use super::TrackShared;

/// A unique identifier for a mixer send track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SendTrackId(pub(crate) Key);

impl From<&SendTrackHandle> for SendTrackId {
	fn from(handle: &SendTrackHandle) -> Self {
		handle.id()
	}
}

pub(crate) struct SendTrack {
	shared: Arc<TrackShared>,
	volume: Parameter<Decibels>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Decibels>>,
	effects: Vec<Box<dyn Effect>>,
	input: [Frame; INTERNAL_BUFFER_SIZE],
}

impl SendTrack {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
	}

	#[must_use]
	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn add_input(&mut self, index: usize, input: Frame) {
		self.input[index] += input;
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
	) {
		let info = Info::new(&clocks.0.resources, &modulators.0.resources);

		for (input_frame, output_frame) in self.input.iter().copied().zip(out.iter_mut()) {
			*output_frame = input_frame;
		}
		self.input.fill(Frame::ZERO);

		// process effects
		for effect in &mut self.effects {
			effect.process(out, dt, &info);
		}

		// apply post-effects volume
		let mut volume_buffer = [Decibels::IDENTITY; INTERNAL_BUFFER_SIZE];
		self.volume
			.update_chunk(&mut volume_buffer[..out.len()], dt, &info);
		for (frame, volume) in out.iter_mut().zip(volume_buffer.iter().copied()) {
			*frame *= volume.as_amplitude();
		}
	}
}

pub(crate) struct SendTrackRoute {
	pub(crate) volume: Parameter<Decibels>,
	pub(crate) set_volume_command_reader: CommandReader<ValueChangeCommand<Decibels>>,
}

impl SendTrackRoute {
	pub fn read_commands(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
	}
}
