use super::{handle::TrackHandle, TrackInput};

#[derive(Clone)]
pub struct TrackRoutes {
	main_track_level: f64,
	sub_track_levels: Vec<(TrackInput, f64)>,
}

impl TrackRoutes {
	pub fn none() -> Self {
		Self {
			main_track_level: 0.0,
			sub_track_levels: vec![],
		}
	}

	pub fn main() -> Self {
		Self {
			main_track_level: 1.0,
			sub_track_levels: vec![],
		}
	}

	pub fn sub(sub_track: &TrackHandle) -> Self {
		Self::none().with_sub_track_level(sub_track, 1.0)
	}

	pub fn with_main_track_level(self, level: f64) -> Self {
		Self {
			main_track_level: level,
			..self
		}
	}

	pub fn with_sub_track_level(mut self, sub_track: &TrackHandle, level: f64) -> Self {
		self.sub_track_levels.push((sub_track.input(), level));
		self
	}

	pub(crate) fn to_vec(mut self, main_track_input: TrackInput) -> Vec<(TrackInput, f64)> {
		self.sub_track_levels
			.push((main_track_input, self.main_track_level));
		self.sub_track_levels
	}
}

impl Default for TrackRoutes {
	fn default() -> Self {
		Self::main()
	}
}
