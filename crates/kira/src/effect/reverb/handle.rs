use crate::{Mix, command::handle_param_setters};

use super::CommandWriters;

/// Controls a reverb effect.
#[derive(Debug)]
pub struct ReverbHandle {
	pub(super) command_writers: CommandWriters,
}

impl ReverbHandle {
	handle_param_setters! {
		/// Sets how much the room reverberates. A higher value will
		/// result in a bigger sounding room. 1.0 gives an infinitely
		/// reverberating room.
		feedback: f64,

		/// Sets how quickly high frequencies disappear from the reverberation.
		damping: f64,

		/// Sets the stereo width of the reverb effect (0.0 being fully mono,
		/// 1.0 being fully stereo).
		stereo_width: f64,

		/// Sets how much dry (unprocessed) signal should be blended
		/// with the wet (processed) signal.
		mix: Mix,
	}
}
