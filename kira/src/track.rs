pub mod routes;
pub mod settings;

use atomic_arena::Index;

use crate::{frame::Frame, manager::resources::parameters::Parameters, value::cached::CachedValue};

use self::settings::TrackSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubTrackId(pub(crate) Index);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackId {
	Main,
	Sub(SubTrackId),
}

impl From<SubTrackId> for TrackId {
	fn from(id: SubTrackId) -> Self {
		Self::Sub(id)
	}
}

pub(crate) struct Track {
	volume: CachedValue,
	panning: CachedValue,
	routes: Vec<(TrackId, CachedValue)>,
	input: Frame,
}

impl Track {
	pub fn new(settings: TrackSettings) -> Self {
		Self {
			volume: CachedValue::new(.., settings.volume, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
			routes: settings.routes.into_vec(),
			input: Frame::from_mono(0.0),
		}
	}

	pub fn routes_mut(&mut self) -> &mut Vec<(TrackId, CachedValue)> {
		&mut self.routes
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn process(&mut self, parameters: &Parameters) -> Frame {
		self.volume.update(parameters);
		self.panning.update(parameters);
		for (_, amount) in &mut self.routes {
			amount.update(parameters);
		}
		let mut output = std::mem::replace(&mut self.input, Frame::from_mono(0.0));
		output *= self.volume.get() as f32;
		output = output.panned(self.panning.get() as f32);
		output
	}
}
