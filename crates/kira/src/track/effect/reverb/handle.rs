use ringbuf::Producer;

use crate::{tween::Tween, CommandError};

use super::Command;

/// Controls a reverb effect.
pub struct ReverbHandle {
	pub(super) command_producer: Producer<Command>,
}

impl ReverbHandle {
	/// Sets how much the room reverberates. A higher value will
	/// result in a bigger sounding room. 1.0 gives an infinitely
	/// reverberating room.
	pub fn set_feedback(&mut self, feedback: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetFeedback(feedback, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how quickly high frequencies disappear from the reverberation.
	pub fn set_damping(&mut self, damping: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetDamping(damping, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	pub fn set_stereo_width(
		&mut self,
		stereo_width: f64,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetStereoWidth(stereo_width, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets how much dry (unprocessed) signal should be blended
	/// with the wet (processed) signal. `0.0` means only the dry
	/// signal will be heard. `1.0` means only the wet signal will
	/// be heard.
	pub fn set_mix(&mut self, mix: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetMix(mix, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}
