use rand::{
	distributions::{
		uniform::{SampleBorrow, SampleUniform, UniformSampler},
		Distribution, Standard,
	},
	Rng,
};

use crate::{tween::Tweenable, Volume};

impl Distribution<Volume> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Volume {
		Volume::Decibels(rng.gen_range(Volume::MIN_DECIBELS..0.0))
	}
}

pub struct UniformVolume {
	low: Volume,
	high: Volume,
	inclusive: bool,
}

impl UniformSampler for UniformVolume {
	type X = Volume;

	fn new<B1, B2>(low: B1, high: B2) -> Self
	where
		B1: SampleBorrow<Self::X> + Sized,
		B2: SampleBorrow<Self::X> + Sized,
	{
		assert!(
			low.borrow()
				.as_amplitude()
				.lt(&high.borrow().as_amplitude()),
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
			low.borrow()
				.as_amplitude()
				.le(&high.borrow().as_amplitude()),
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

impl SampleUniform for Volume {
	type Sampler = UniformVolume;
}
