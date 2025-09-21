use std::time::Duration;

use crate::{Decibels, Mix, command::handle_param_setters};

use super::CommandWriters;

/// Controls a compressor.
#[derive(Debug)]
pub struct CompressorHandle {
	pub(super) command_writers: CommandWriters,
}

impl CompressorHandle {
	handle_param_setters! {
		/// Sets the volume above which volume will start to be decreased (in decibels).
		threshold: f64,

		/// Sets how much the signal will be compressed.
		///
		/// A ratio of `2.0` (or 2 to 1) means an increase of 3dB will
		/// become an increase of 1.5dB. Ratios between `0.0` and `1.0`
		/// will actually expand the audio.
		ratio: f64,

		/// Sets how much time it takes for the volume attenuation to ramp up once
		/// the input volume exceeds the threshold.
		attack_duration: Duration,

		/// Sets how much time it takes for the volume attenuation to relax once
		/// the input volume dips below the threshold.
		release_duration: Duration,

		/// Sets the amount to change the volume after processing (in dB).
		///
		/// This can be used to compensate for the decrease in volume resulting
		/// from compression. This is only applied to the wet signal, nto the
		/// dry signal.
		makeup_gain: Decibels,

		/// Sets how much dry (unprocessed) signal should be blended
		/// with the wet (processed) signal.
		mix: Mix,
	}
}
