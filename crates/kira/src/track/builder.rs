use crate::{tween::Value, Volume};

use super::{effect::EffectBuilder, routes::TrackRoutes, Effect};

/// Configures a mixer track.
#[non_exhaustive]
pub struct TrackBuilder {
	/// The volume of the track.
	pub(crate) volume: Value<Volume>,
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
			volume: Value::Fixed(Volume::Amplitude(1.0)),
			routes: TrackRoutes::new(),
			effects: vec![],
		}
	}

	/// Sets the volume of the track.
	pub fn volume(self, volume: impl Into<Value<Volume>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets how the output of this track should be routed
	/// to other mixer tracks.
	pub fn routes(self, routes: TrackRoutes) -> Self {
		Self { routes, ..self }
	}

	/// Adds an effect to the track.
	pub fn add_effect<B: EffectBuilder>(&mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.effects.push(effect);
		handle
	}

	/// Adds an effect to the track and returns the [`TrackBuilder`].
	///
	/// If you need to modify the effect later, use [`add_effect`](Self::add_effect),
	/// which returns the effect handle.
	pub fn with_effect<B: EffectBuilder>(mut self, builder: B) -> Self {
		self.add_effect(builder);
		self
	}
}

impl Default for TrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}
