use super::TrackInput;

pub struct TrackHandle {
	input: TrackInput,
}

impl TrackHandle {
	pub(crate) fn new(input: TrackInput) -> Self {
		Self { input }
	}

	pub(crate) fn input(&self) -> TrackInput {
		self.input.clone()
	}
}
