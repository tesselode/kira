use super::{routes::TrackRoutes, Effect};

/// Settings for a mixer track.
#[non_exhaustive]
pub struct TrackSettings {
	/// The volume of the track.
	pub volume: f64,
	/// The panning of the track, where 0 is hard left
	/// and 1 is hard right.
	pub panning: f64,
	/// How the output of this track should be routed
	/// to other mixer tracks.
	pub routes: TrackRoutes,
	/// The effects that should be applied to the input audio
	/// for this track.
	pub effects: Vec<Box<dyn Effect>>,
}

impl TrackSettings {
	/// Creates a new [`TrackSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			volume: 1.0,
			panning: 0.5,
			routes: TrackRoutes::new(),
			effects: vec![],
		}
	}

	/// Sets the volume of the track.
	pub fn volume(self, volume: f64) -> Self {
		Self { volume, ..self }
	}

	/// Sets the panning of the track, where 0 is hard left
	/// and 1 is hard right.
	pub fn panning(self, panning: f64) -> Self {
		Self { panning, ..self }
	}

	/// Sets how the output of this track should be routed
	/// to other mixer tracks.
	pub fn routes(self, routes: TrackRoutes) -> Self {
		Self { routes, ..self }
	}

	/// Adds an effect to the track.
	pub fn with_effect(mut self, effect: impl Effect + 'static) -> Self {
		self.effects.push(Box::new(effect));
		self
	}
}

impl Default for TrackSettings {
	fn default() -> Self {
		Self::new()
	}
}
