use crate::{sound_bank::SoundBank, stereo_sample::StereoSample};

pub struct Backend {
	sample_rate: u32,
	sound_bank: SoundBank,
}

impl Backend {
	pub fn new(sample_rate: u32, sound_bank: SoundBank) -> Self {
		Self {
			sample_rate,
			sound_bank,
		}
	}

	pub fn process(&mut self) -> StereoSample {
		StereoSample::from_mono(0.0)
	}
}
