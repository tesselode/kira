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
	Decibels, Frame, Parameter,
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
	input: Vec<Frame>,
	internal_buffer_size: usize,
}

impl SendTrack {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate, self.internal_buffer_size);
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

	pub fn add_input(&mut self, input: &[Frame], volume: Decibels) {
		for (input, added) in self.input.iter_mut().zip(input.iter().copied()) {
			*input += added * volume.as_amplitude();
		}
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(&mut self, out: &mut [Frame], dt: f64, info: &Info) {
		self.volume.update(dt * out.len() as f64, info);
		for (out_frame, input_frame) in out.iter_mut().zip(self.input.iter().copied()) {
			*out_frame += input_frame;
		}
		self.input.fill(Frame::ZERO);
		for effect in &mut self.effects {
			effect.process(out, dt, info);
		}
		let num_frames = out.len();
		for (i, frame) in out.iter_mut().enumerate() {
			let time_in_chunk = (i + 1) as f64 / num_frames as f64;
			let volume = self.volume.interpolated_value(time_in_chunk).as_amplitude();
			*frame *= volume;
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
