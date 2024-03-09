use std::{collections::HashMap, sync::Arc};

use crate::{
	command::command_writer_and_reader,
	dsp::Frame,
	tween::{Parameter, Value},
	Volume,
};

use super::{
	effect::EffectBuilder, routes::TrackRoutes, Effect, Route, Track, TrackHandle, TrackId,
	TrackShared,
};

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

	/**
	Sets the volume of the track.

	# Examples

	Set the volume as a factor:

	```
	# use kira::track::TrackBuilder;
	let builder = TrackBuilder::new().volume(0.5);
	```

	Set the volume as a gain in decibels:

	```
	# use kira::track::TrackBuilder;
	let builder = TrackBuilder::new().volume(kira::Volume::Decibels(-6.0));
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		track::TrackBuilder,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let builder = TrackBuilder::new().volume(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
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

	/**
	Adds an effect to the track.

	# Examples

	```
	use kira::track::{TrackBuilder, effect::delay::DelayBuilder};

	let mut builder = TrackBuilder::new();
	let delay_handle = builder.add_effect(DelayBuilder::new());
	```
	*/
	pub fn add_effect<B: EffectBuilder>(&mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.effects.push(effect);
		handle
	}

	/**
	Adds an effect to the track and returns the [`TrackBuilder`].

	If you need to modify the effect later, use [`add_effect`](Self::add_effect),
	which returns the effect handle.

	# Examples

	```
	use kira::track::{
		TrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder},
	};

	let mut builder = TrackBuilder::new()
		.with_effect(FilterBuilder::new())
		.with_effect(ReverbBuilder::new());
	```
	*/
	pub fn with_effect<B: EffectBuilder>(mut self, builder: B) -> Self {
		self.add_effect(builder);
		self
	}

	pub(crate) fn build(self, id: TrackId) -> (Track, TrackHandle) {
		let shared = Arc::new(TrackShared::new());
		let (volume_change_command_writer, volume_change_command_reader) =
			command_writer_and_reader();
		let mut route_volume_change_command_writers = HashMap::new();
		let mut routes = vec![];
		for (id, volume) in &self.routes.0 {
			let (writer, reader) = command_writer_and_reader();
			route_volume_change_command_writers.insert(*id, writer);
			routes.push(Route {
				destination: *id,
				volume: Parameter::new(*volume, Volume::Amplitude(1.0)),
				volume_change_command_reader: reader,
			});
		}
		let track = Track {
			shared: shared.clone(),
			volume: Parameter::new(self.volume, Volume::Amplitude(1.0)),
			volume_change_command_reader,
			routes,
			effects: self.effects,
			input: Frame::ZERO,
		};
		let handle = TrackHandle {
			id,
			shared,
			volume_change_command_writer,
			route_volume_change_command_writers,
		};
		(track, handle)
	}
}

impl Default for TrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}
