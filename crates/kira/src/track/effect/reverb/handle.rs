use ringbuf::HeapProducer;

use crate::{
	tween::{Tween, Value},
	CommandError,
};

use super::Command;

/// Controls a reverb effect.
pub struct ReverbHandle {
	pub(super) command_producer: HeapProducer<Command>,
}

impl ReverbHandle {
	/// Sets how much the room reverberates. A higher value will
	/// result in a bigger sounding room. 1.0 gives an infinitely
	/// reverberating room.
	pub fn set_feedback(
		&mut self,
		feedback: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFeedback(feedback.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how quickly high frequencies disappear from the reverberation.
	pub fn set_damping(
		&mut self,
		damping: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetDamping(damping.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	pub fn set_stereo_width(
		&mut self,
		stereo_width: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetStereoWidth(stereo_width.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn set_mix(
		&mut self,
		mix: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
