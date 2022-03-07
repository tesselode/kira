use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterDistances {
	pub min_distance: f32,
	pub max_distance: f32,
}

impl EmitterDistances {
	pub(crate) fn relative_distance(&self, distance: f32) -> f32 {
		let distance = distance.clamp(self.min_distance, self.max_distance);
		(distance - self.min_distance) / (self.max_distance - self.min_distance)
	}
}

impl Default for EmitterDistances {
	fn default() -> Self {
		Self {
			min_distance: 1.0,
			max_distance: 500.0,
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
