use crate::command::handle_param_setters;

use super::CommandWriters;

/// Controls a reverb effect.
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
		/// with the wet (processed) signal. `0.0` means only the dry
		/// signal will be heard. `1.0` means only the wet signal will
		/// be heard.
		mix: f64,
	}
}
