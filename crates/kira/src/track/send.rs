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
	tween::Parameter,
	Dbfs, Frame,
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
	volume: Parameter<Dbfs>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Dbfs>>,
	effects: Vec<Box<dyn Effect>>,
	input: Frame,
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

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(&mut self, dt: f64, info: &Info) -> Frame {
		self.volume.update(dt, info);
		let mut output = std::mem::replace(&mut self.input, Frame::ZERO);
		for effect in &mut self.effects {
			output = effect.process(output, dt, info);
		}
		output * self.volume.value().as_amplitude()
	}
}

pub(crate) struct SendTrackRoute {
	pub(crate) volume: Parameter<Dbfs>,
	pub(crate) set_volume_command_reader: CommandReader<ValueChangeCommand<Dbfs>>,
}

impl SendTrackRoute {
	pub fn read_commands(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
	}
}
