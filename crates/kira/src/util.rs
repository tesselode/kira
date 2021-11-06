//! Useful functions for writing [`Effect`s](crate::track::Effect)
//! and [`AudioStream`s](crate::audio_stream::AudioStream).

use crate::dsp::Frame;

/// Given a previous frame, a current frame, the two next frames,
/// and a position `x` from 0.0 to 1.0 between the current frame
/// and next frame, get an approximated frame.
///
/// This is the 4-point, 3rd-order Hermite interpolation x-form
/// algorithm from "Polynomial Interpolators for High-Quality
/// Resampling of Oversampled Audio" by Olli Niemitalo, p. 43:
/// http://yehar.com/blog/wp-content/uploads/2009/08/deip.pdf
pub fn interpolate_frame(
	previous: Frame,
	current: Frame,
	next_1: Frame,
	next_2: Frame,
	fraction: f32,
) -> Frame {
	let c0 = current;
	let c1 = (next_1 - previous) * 0.5;
	let c2 = previous - current * 2.5 + next_1 * 2.0 - next_2 * 0.5;
	let c3 = (next_2 - previous) * 0.5 + (current - next_1) * 1.5;
	((c3 * fraction + c2) * fraction + c1) * fraction + c0
}
