use std::{collections::HashMap, ops::RangeInclusive, sync::Arc};

use glam::Vec3;

use crate::{
	command::command_writer_and_reader,
	effect::EffectBuilder,
	listener::ListenerId,
	manager::backend::{resources::ResourceStorage, RendererShared},
	tween::{Easing, Parameter, Value},
	Volume,
};

use super::{
	Effect, SendTrackId, SendTrackRoute, SpatialData, SpatialTrackHandle, Track, TrackShared,
};

/// Configures a spatial mixer track.
pub struct SpatialTrackBuilder {
	/// The volume of the track.
	pub(crate) volume: Value<Volume>,
	/// The effects that should be applied to the input audio
	/// for this track.
	pub(crate) effects: Vec<Box<dyn Effect>>,
	/// The number of child tracks that can be added to this track.
	pub(crate) sub_track_capacity: u16,
	/// The maximum number of sounds that can be played simultaneously on this track.
	pub(crate) sound_capacity: u16,
	pub(crate) sends: HashMap<SendTrackId, Value<Volume>>,
	/// The distances from a listener at which the track is loudest and quietest.
	pub(crate) distances: SpatialTrackDistances,
	/// How the track's volume will change with distance.
	///
	/// If `None`, the track will output at a constant volume.
	pub(crate) attenuation_function: Option<Easing>,
	/// Whether the track's output should be panned left or right depending on its
	/// direction from the listener.
	pub(crate) enable_spatialization: bool,
}

impl SpatialTrackBuilder {
	/// Creates a new [`TrackBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(Volume::Amplitude(1.0)),
			effects: vec![],
			sub_track_capacity: 16,
			sound_capacity: 128,
			sends: HashMap::new(),
			distances: SpatialTrackDistances::default(),
			attenuation_function: Some(Easing::Linear),
			enable_spatialization: true,
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
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn volume(self, volume: impl Into<Value<Volume>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the maximum number of sub-tracks this track can have.
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn sub_track_capacity(self, capacity: u16) -> Self {
		Self {
			sub_track_capacity: capacity,
			..self
		}
	}

	/// Sets the maximum number of sounds that can be played simultaneously on this track.
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn sound_capacity(self, capacity: u16) -> Self {
		Self {
			sound_capacity: capacity,
			..self
		}
	}

	pub fn with_send(
		mut self,
		track: impl Into<SendTrackId>,
		volume: impl Into<Value<Volume>>,
	) -> Self {
		self.sends.insert(track.into(), volume.into());
		self
	}

	/**
	Adds an effect to the track.

	# Examples

	```
	use kira::{track::TrackBuilder, effect::delay::DelayBuilder};

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
	use kira::{
		track::TrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder},
	};

	let mut builder = TrackBuilder::new()
		.with_effect(FilterBuilder::new())
		.with_effect(ReverbBuilder::new());
	```
	*/
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn with_effect<B: EffectBuilder>(mut self, builder: B) -> Self {
		self.add_effect(builder);
		self
	}

	/** Adds an already built effect into this track.

	`Box<dyn Effect>` values are created when calling `build` on an effect builder, which gives you
	an effect handle, as well as this boxed effect, which is the actual audio effect.

	This is a lower-level method than [`Self::add_effect`], and you should probably use it rather
	than this method, unless you have a reason to.

	# Examples

	```
	use kira::track::TrackBuilder;
	use kira::effect::{EffectBuilder, delay::DelayBuilder};

	let mut builder = TrackBuilder::new();
	let delay_builder = DelayBuilder::new();
	let (effect, delay_handle) = delay_builder.build();
	let delay_handle = builder.add_built_effect(effect);
	```
	*/
	pub fn add_built_effect(&mut self, effect: Box<dyn Effect>) {
		self.effects.push(effect);
	}

	/** Add an already-built effect and return the [`TrackBuilder`].

	 `Box<dyn Effect>` values are created when calling `build` on an effect builder, which gives you
	 an effect handle, as well as this boxed effect, which is the actual audio effect.

	 This is a lower-level method than [`Self::with_effect`], and you should probably use it rather
	 than this method, unless you have a reason to.

	# Examples

	```
	use kira::{
		track::TrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder, EffectBuilder},
	};

	let (filter_effect, filter_handle) = FilterBuilder::new().build();
	let (reverb_effect, reverb_handle) = ReverbBuilder::new().build();
	let mut builder = TrackBuilder::new()
		.with_built_effect(filter_effect)
		.with_built_effect(reverb_effect);
	```
	 */
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn with_built_effect(mut self, effect: Box<dyn Effect>) -> Self {
		self.add_built_effect(effect);
		self
	}

	/// Sets the distances from a listener at which the emitter is loudest and quietest.
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn distances(self, distances: impl Into<SpatialTrackDistances>) -> Self {
		Self {
			distances: distances.into(),
			..self
		}
	}

	/// Sets how the emitter's volume will change with distance.
	///
	/// If `None`, the emitter will output at a constant volume.
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn attenuation_function(self, attenuation_function: impl Into<Option<Easing>>) -> Self {
		Self {
			attenuation_function: attenuation_function.into(),
			..self
		}
	}

	/// Sets whether the emitter's output should be panned left or right depending on its
	/// direction from the listener.
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn enable_spatialization(self, enable_spatialization: bool) -> Self {
		Self {
			enable_spatialization,
			..self
		}
	}

	#[must_use]
	pub(crate) fn build(
		self,
		renderer_shared: Arc<RendererShared>,
		listener_id: ListenerId,
		position: Value<Vec3>,
	) -> (Track, SpatialTrackHandle) {
		let (set_volume_command_writer, set_volume_command_reader) = command_writer_and_reader();
		let (set_position_command_writer, set_position_command_reader) =
			command_writer_and_reader();
		let shared = Arc::new(TrackShared::new());
		let (sounds, sound_controller) = ResourceStorage::new(self.sound_capacity);
		let (sub_tracks, sub_track_controller) = ResourceStorage::new(self.sub_track_capacity);
		let mut sends = vec![];
		let mut send_volume_command_writers = HashMap::new();
		for (send_track_id, volume) in self.sends {
			let (set_volume_command_writer, set_volume_command_reader) =
				command_writer_and_reader();
			sends.push((
				send_track_id,
				SendTrackRoute {
					volume: Parameter::new(volume, Volume::Amplitude(1.0)),
					set_volume_command_reader,
				},
			));
			send_volume_command_writers.insert(send_track_id, set_volume_command_writer);
		}
		let track = Track {
			shared: shared.clone(),
			volume: Parameter::new(self.volume, Volume::Amplitude(1.0)),
			set_volume_command_reader,
			sounds,
			sub_tracks,
			effects: self.effects,
			sends,
			spatial_data: Some(SpatialData {
				listener_id,
				position: Parameter::new(position, Vec3::ZERO),
				set_position_command_reader,
				distances: self.distances,
				attenuation_function: self.attenuation_function,
				enable_spatialization: self.enable_spatialization,
			}),
		};
		let handle = SpatialTrackHandle {
			renderer_shared,
			shared: Some(shared),
			set_volume_command_writer,
			sound_controller,
			sub_track_controller,
			send_volume_command_writers,
			set_position_command_writer,
		};
		(track, handle)
	}
}

impl Default for SpatialTrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// The distances from a listener at which an emitter is loudest and quietest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialTrackDistances {
	/// The distance from a listener at which an emitter outputs at full volume.
	pub min_distance: f32,
	/// The distance from a listener at which an emitter becomes inaudible.
	pub max_distance: f32,
}

impl SpatialTrackDistances {
	#[must_use]
	pub(crate) fn relative_distance(&self, distance: f32) -> f32 {
		let distance = distance.clamp(self.min_distance, self.max_distance);
		(distance - self.min_distance) / (self.max_distance - self.min_distance)
	}
}

impl Default for SpatialTrackDistances {
	fn default() -> Self {
		Self {
			min_distance: 1.0,
			max_distance: 100.0,
		}
	}
}

impl From<(f32, f32)> for SpatialTrackDistances {
	fn from((min_distance, max_distance): (f32, f32)) -> Self {
		Self {
			min_distance,
			max_distance,
		}
	}
}

impl From<[f32; 2]> for SpatialTrackDistances {
	fn from([min_distance, max_distance]: [f32; 2]) -> Self {
		Self {
			min_distance,
			max_distance,
		}
	}
}

impl From<RangeInclusive<f32>> for SpatialTrackDistances {
	fn from(range: RangeInclusive<f32>) -> Self {
		Self {
			min_distance: *range.start(),
			max_distance: *range.end(),
		}
	}
}