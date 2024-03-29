use crate::{
	spatial::emitter::{EmitterHandle, EmitterId},
	track::{SubTrackId, TrackHandle, TrackId},
};

/// Where a source of audio should be routed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputDestination {
	/// A mixer track.
	Track(TrackId),
	/// An emitter in a spatial scene.
	Emitter(EmitterId),
}

impl OutputDestination {
	/// Route audio to the main mixer track.
	pub const MAIN_TRACK: Self = Self::Track(TrackId::Main);
}

impl Default for OutputDestination {
	fn default() -> Self {
		Self::MAIN_TRACK
	}
}

impl From<TrackId> for OutputDestination {
	fn from(v: TrackId) -> Self {
		Self::Track(v)
	}
}

impl From<SubTrackId> for OutputDestination {
	fn from(id: SubTrackId) -> Self {
		Self::Track(TrackId::Sub(id))
	}
}

impl From<&TrackHandle> for OutputDestination {
	fn from(handle: &TrackHandle) -> Self {
		Self::Track(handle.id())
	}
}

impl From<EmitterId> for OutputDestination {
	fn from(v: EmitterId) -> Self {
		Self::Emitter(v)
	}
}

impl From<&EmitterHandle> for OutputDestination {
	fn from(handle: &EmitterHandle) -> Self {
		Self::Emitter(handle.id())
	}
}
