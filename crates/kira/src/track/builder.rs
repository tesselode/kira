use super::{effect::EffectBuilder, routes::TrackRoutes, Effect};

/// Configures a mixer track.
#[non_exhaustive]
pub struct TrackBuilder {
	/// The volume of the track.
	pub(crate) volume: f64,
	/// The panning of the track, where 0 is hard left
	/// and 1 is hard right.
	pub(crate) panning: f64,
	/// How the output of this track should be routed
	/// to other mixer tracks.
	pub(crate) routes: TrackRoutes,
	/// The effects that should be applied to the input audio
	/// for this track.
	pub(crate) effects: Vec<Box<dyn Effect>>,
}

impl TrackBuilder {
	/// Creates a new [`TrackBuilder`] with the default settings.
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
	pub fn add_effect<B: EffectBuilder>(mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.effects.push(effect);
		handle
	}
}

impl Default for TrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}
