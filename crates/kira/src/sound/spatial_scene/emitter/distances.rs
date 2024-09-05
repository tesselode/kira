use std::ops::RangeInclusive;

/// The distances from a listener at which an emitter is loudest and quietest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterDistances {
	/// The distance from a listener at which an emitter outputs at full volume.
	pub min_distance: f32,
	/// The distance from a listener at which an emitter becomes inaudible.
	pub max_distance: f32,
}

impl EmitterDistances {
	#[must_use]
	pub(crate) fn relative_distance(&self, distance: f32) -> f32 {
		let distance = distance.clamp(self.min_distance, self.max_distance);
		(distance - self.min_distance) / (self.max_distance - self.min_distance)
	}
}

impl Default for EmitterDistances {
	fn default() -> Self {
		Self {
			min_distance: 1.0,
			max_distance: 100.0,
		}
	}
}

impl From<(f32, f32)> for EmitterDistances {
	fn from((min_distance, max_distance): (f32, f32)) -> Self {
		Self {
			min_distance,
			max_distance,
		}
	}
}

impl From<[f32; 2]> for EmitterDistances {
	fn from([min_distance, max_distance]: [f32; 2]) -> Self {
		Self {
			min_distance,
			max_distance,
		}
	}
}

impl From<RangeInclusive<f32>> for EmitterDistances {
	fn from(range: RangeInclusive<f32>) -> Self {
		Self {
			min_distance: *range.start(),
			max_distance: *range.end(),
		}
	}
}
