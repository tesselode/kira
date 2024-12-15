use std::{collections::HashMap, sync::Arc};

use crate::{
	command::command_writer_and_reader,
	effect::EffectBuilder,
	manager::backend::{resources::ResourceStorage, RendererShared},
	playback_state_manager::PlaybackStateManager,
	tween::{Parameter, Value},
	Decibels, Frame, INTERNAL_BUFFER_SIZE,
};

use super::{
	command_writers_and_readers, Effect, SendTrackId, SendTrackRoute, Track, TrackHandle,
	TrackShared,
};

/// Configures a mixer track.
pub struct TrackBuilder {
	/// The volume of the track.
	pub(crate) volume: Value<Decibels>,
	/// The effects that should be applied to the input audio
	/// for this track.
	pub(crate) effects: Vec<Box<dyn Effect>>,
	/// The number of child tracks that can be added to this track.
	pub(crate) sub_track_capacity: u16,
	/// The maximum number of sounds that can be played simultaneously on this track.
	pub(crate) sound_capacity: u16,
	pub(crate) sends: HashMap<SendTrackId, Value<Decibels>>,
	pub(crate) persist_until_sounds_finish: bool,
}

impl TrackBuilder {
	/// Creates a new [`TrackBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(Decibels::IDENTITY),
			effects: vec![],
			sub_track_capacity: 128,
			sound_capacity: 128,
			sends: HashMap::new(),
			persist_until_sounds_finish: false,
		}
	}

	/**
	Sets the volume of the track.

	# Examples

	Set the volume to a fixed decibel value:

	```
	# use kira::track::TrackBuilder;
	let builder = TrackBuilder::new().volume(-6.0);
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		track::TrackBuilder,
		tween::{Easing, Value, Mapping},
		Decibels,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let builder = TrackBuilder::new().volume(Value::FromModulator {
		id: tweener.id(),
		mapping: Mapping {
			input_range: (0.0, 1.0),
			output_range: (Decibels::SILENCE, Decibels::IDENTITY),
			easing: Easing::Linear,
		},
	});
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method consumes self and returns a modified TrackBuilder, so the return value should be used"]
	pub fn volume(self, volume: impl Into<Value<Decibels>>) -> Self {
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

	/// Routes this track to the given send track with the given volume.
	pub fn with_send(
		mut self,
		track: impl Into<SendTrackId>,
		volume: impl Into<Value<Decibels>>,
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

	/// Sets whether the track should stay alive while sounds are playing on it.
	///
	/// By default, as soon as a track's handle is dropped, the track is unloaded.
	/// If this is set to `true`, the track will wait until all sounds on the track
	/// are finished before unloading.
	pub fn persist_until_sounds_finish(self, persist: bool) -> Self {
		Self {
			persist_until_sounds_finish: persist,
			..self
		}
	}

	#[must_use]
	pub(crate) fn build(self, renderer_shared: Arc<RendererShared>) -> (Track, TrackHandle) {
		let (command_writers, command_readers) = command_writers_and_readers();
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
					volume: Parameter::new(volume, Decibels::IDENTITY),
					set_volume_command_reader,
				},
			));
			send_volume_command_writers.insert(send_track_id, set_volume_command_writer);
		}
		let track = Track {
			shared: shared.clone(),
			command_readers,
			volume: Parameter::new(self.volume, Decibels::IDENTITY),
			sounds,
			sub_tracks,
			effects: self.effects,
			sends,
			persist_until_sounds_finish: self.persist_until_sounds_finish,
			spatial_data: None,
			playback_state_manager: PlaybackStateManager::new(None),
			temp_buffer: vec![Frame::ZERO; INTERNAL_BUFFER_SIZE],
		};
		let handle = TrackHandle {
			renderer_shared,
			shared,
			command_writers,
			sound_controller,
			sub_track_controller,
			send_volume_command_writers,
		};
		(track, handle)
	}
}

impl Default for TrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}
