use crate::{
	command::ValueChangeCommand,
	tween::{Tween, Value},
};

use super::CommandWriters;

/// Controls a panning control effect.
pub struct PanningControlHandle {
	pub(super) command_writers: CommandWriters,
}

impl PanningControlHandle {
	/// Sets the panning adjustment to apply to input audio.
	pub fn set_panning(&mut self, panning: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.panning_change
			.write(ValueChangeCommand {
				target: panning.into(),
				tween,
			})
	}
}
