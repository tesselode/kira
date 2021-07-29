pub mod routes;
pub mod settings;

use atomic_arena::Index;

use crate::{frame::Frame, value::cached::CachedValue};

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
}
