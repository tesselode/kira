use all_pass::AllPassFilter;
use comb::CombFilter;

use crate::{CachedValue, Frame, Value};

use super::Effect;

mod all_pass;
mod comb;

const NUM_COMB_FILTERS: usize = 8;
const NUM_ALL_PASS_FILTERS: usize = 4;
const GAIN: f32 = 0.015;
const STEREO_SPREAD: usize = 23;

/// Settings for a `Reverb`.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct ReverbSettings {
	/// The size of the simulated room.
	room_size: Value<f64>,
	/// How quickly high frequencies disappear from the reverberation.
	damping: Value<f64>,
	/// The stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	stereo_width: Value<f64>,
}

impl ReverbSettings {
	/// Creates a new `ReverbSettings` with the default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the size of the simulated room.
	pub fn room_size(self, room_size: impl Into<Value<f64>>) -> Self {
		Self {
			room_size: room_size.into(),
			..self
		}
	}

	/// Sets how quickly high frequencies disappear from the reverberation.
	pub fn damping(self, damping: impl Into<Value<f64>>) -> Self {
		Self {
			damping: damping.into(),
			..self
		}
	}

	/// Sets the stereo width of the reverb effect (0.0 being fully mono,
	/// 1.0 being fully stereo).
	pub fn stereo_width(self, stereo_width: impl Into<Value<f64>>) -> Self {
		Self {
			stereo_width: stereo_width.into(),
			..self
		}
	}
}

impl Default for ReverbSettings {
	fn default() -> Self {
		Self {
			room_size: Value::Fixed(0.9),
			damping: Value::Fixed(0.1),
			stereo_width: Value::Fixed(1.0),
		}
	}
}

/// A reverb effect. Useul for simulating room tones.
// This code is based on Freeverb by Jezar at Dreampoint, found here:
// http://blog.bjornroche.com/2012/06/freeverb-original-public-domain-code-by.html
#[derive(Debug)]
pub struct Reverb {
	room_size: CachedValue<f64>,
	damping: CachedValue<f64>,
	stereo_width: CachedValue<f64>,

	comb_filters: [(CombFilter, CombFilter); NUM_COMB_FILTERS],
	all_pass_filters: [(AllPassFilter, AllPassFilter); NUM_ALL_PASS_FILTERS],
}

impl Reverb {
	/// Creates a new `Reverb` effect.
	pub fn new(settings: ReverbSettings) -> Self {
		Self {
			room_size: CachedValue::new(settings.room_size, 0.9),
			damping: CachedValue::new(settings.damping, 0.1),
			stereo_width: CachedValue::new(settings.stereo_width, 1.0),
			comb_filters: [
				(CombFilter::new(1116), CombFilter::new(1116 + STEREO_SPREAD)),
				(CombFilter::new(1188), CombFilter::new(1188 + STEREO_SPREAD)),
				(CombFilter::new(1277), CombFilter::new(1277 + STEREO_SPREAD)),
				(CombFilter::new(1356), CombFilter::new(1356 + STEREO_SPREAD)),
				(CombFilter::new(1422), CombFilter::new(1422 + STEREO_SPREAD)),
				(CombFilter::new(1491), CombFilter::new(1491 + STEREO_SPREAD)),
				(CombFilter::new(1557), CombFilter::new(1557 + STEREO_SPREAD)),
				(CombFilter::new(1617), CombFilter::new(1617 + STEREO_SPREAD)),
			],
			all_pass_filters: [
				(
					AllPassFilter::new(556),
					AllPassFilter::new(556 + STEREO_SPREAD),
				),
				(
					AllPassFilter::new(441),
					AllPassFilter::new(441 + STEREO_SPREAD),
				),
				(
					AllPassFilter::new(341),
					AllPassFilter::new(341 + STEREO_SPREAD),
				),
				(
					AllPassFilter::new(225),
					AllPassFilter::new(225 + STEREO_SPREAD),
				),
			],
		}
	}
}

impl Effect for Reverb {
	fn process(
		&mut self,
		_dt: f64,
		input: crate::Frame,
		parameters: &crate::parameter::Parameters,
	) -> crate::Frame {
		self.room_size.update(parameters);
		self.damping.update(parameters);
		self.stereo_width.update(parameters);

		let room_size = self.room_size.value() as f32;
		let damping = self.damping.value() as f32;
		let stereo_width = self.stereo_width.value() as f32;

		let mut output = Frame::from_mono(0.0);
		let input = (input.left + input.right) * GAIN;
		// accumulate comb filters in parallel
		for comb_filter in &mut self.comb_filters {
			output.left += comb_filter.0.process(input, room_size, damping);
			output.right += comb_filter.1.process(input, room_size, damping);
		}
		// feed through all-pass filters in series
		for all_pass_filter in &mut self.all_pass_filters {
			output.left = all_pass_filter.0.process(output.left);
			output.right = all_pass_filter.1.process(output.right);
		}
		let wet_1 = stereo_width / 2.0 + 0.5;
		let wet_2 = (1.0 - stereo_width) / 2.0;
		Frame::new(
			output.left * wet_1 + output.right * wet_2,
			output.right * wet_1 + output.left * wet_2,
		)
	}
}
