use rand::{
	distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler},
	Rng,
};

use crate::tween::Tweenable;

use super::PlaybackRate;

pub struct UniformPlaybackRate {
	low: PlaybackRate,
	high: PlaybackRate,
	inclusive: bool,
}

impl UniformSampler for UniformPlaybackRate {
	type X = PlaybackRate;

	fn new<B1, B2>(low: B1, high: B2) -> Self
	where
		B1: SampleBorrow<Self::X> + Sized,
		B2: SampleBorrow<Self::X> + Sized,
	{
		assert!(
			low.borrow().as_factor().lt(&high.borrow().as_factor()),
			"Uniform::new called with `low >= high`"
		);
		Self {
			low: *low.borrow(),
			high: *high.borrow(),
			inclusive: false,
		}
	}

	fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
	where
		B1: SampleBorrow<Self::X> + Sized,
		B2: SampleBorrow<Self::X> + Sized,
	{
		assert!(
			low.borrow().as_factor().le(&high.borrow().as_factor()),
			"Uniform::new called with `low > high`"
		);
		Self {
			low: *low.borrow(),
			high: *high.borrow(),
			inclusive: true,
		}
	}

	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
		Tweenable::interpolate(
			self.low,
			self.high,
			if self.inclusive {
				rng.gen_range(0.0..=1.0)
			} else {
				rng.gen_range(0.0..1.0)
			},
		)
	}
}

impl SampleUniform for PlaybackRate {
	type Sampler = UniformPlaybackRate;
}
