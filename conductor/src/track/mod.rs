pub mod effect_slot;
pub mod id;
pub mod index;

use crate::stereo_sample::StereoSample;

use self::effect_slot::EffectSlot;

pub struct TrackSettings {
	pub volume: f64,
}

impl Default for TrackSettings {
	fn default() -> Self {
		Self { volume: 1.0 }
	}
}

pub(crate) struct Track {
	volume: f64,
	effect_slots: Vec<EffectSlot>,
	input: StereoSample,
}

impl Track {
	pub fn new(settings: TrackSettings) -> Self {
		Self {
			volume: settings.volume,
			effect_slots: vec![],
			input: StereoSample::from_mono(0.0),
		}
	}

	pub fn add_input(&mut self, input: StereoSample) {
		self.input += input;
	}

	pub fn process(&mut self, dt: f64) -> StereoSample {
		let mut input = self.input;
		self.input = StereoSample::from_mono(0.0);
		for effect_slot in &mut self.effect_slots {
			input = effect_slot.process(dt, input);
		}
		input * (self.volume as f32)
	}
}
