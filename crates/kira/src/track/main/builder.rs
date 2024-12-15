use crate::{
	command::command_writer_and_reader,
	effect::{Effect, EffectBuilder},
	resources::ResourceStorage,
	tween::Parameter,
	Decibels, Value,
};

use super::{MainTrack, MainTrackHandle};

/// Configures the main mixer track.
pub struct MainTrackBuilder {
	/// The volume of the track.
	pub(crate) volume: Value<Decibels>,
	/// The effects that should be applied to the input audio
	/// for this track.
	pub(crate) effects: Vec<Box<dyn Effect>>,
	/// The maximum number of sounds that can be played simultaneously on this track.
	pub(crate) sound_capacity: u16,
}

impl MainTrackBuilder {
	/// Creates a new [`MainTrackBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(Decibels::IDENTITY),
			effects: vec![],
			sound_capacity: 128,
		}
	}

	/// Sets the volume of the main mixer track.
	#[must_use = "This method consumes self and returns a modified MainTrackBuilder, so the return value should be used"]
	pub fn volume(self, volume: impl Into<Value<Decibels>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/// Sets the maximum number of sounds that can be played simultaneously on this track.
	#[must_use = "This method consumes self and returns a modified MainTrackBuilder, so the return value should be used"]
	pub fn sound_capacity(self, capacity: u16) -> Self {
		Self {
			sound_capacity: capacity,
			..self
		}
	}

	/**
	Adds an effect to the track.

	# Examples

	```
	use kira::{track::MainTrackBuilder, effect::delay::DelayBuilder};

	let mut builder = MainTrackBuilder::new();
	let delay_handle = builder.add_effect(DelayBuilder::new());
	```
	*/
	pub fn add_effect<B: EffectBuilder>(&mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.effects.push(effect);
		handle
	}

	/**
	Adds an effect to the track and returns the [`MainTrackBuilder`].

	If you need to modify the effect later, use [`add_effect`](Self::add_effect),
	which returns the effect handle.

	# Examples

	```
	use kira::{
		track::MainTrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder},
	};

	let mut builder = MainTrackBuilder::new()
		.with_effect(FilterBuilder::new())
		.with_effect(ReverbBuilder::new());
	```
	*/
	#[must_use = "This method consumes self and returns a modified MainTrackBuilder, so the return value should be used"]
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
	use kira::track::MainTrackBuilder;
	use kira::effect::{EffectBuilder, delay::DelayBuilder};

	let mut builder = MainTrackBuilder::new();
	let delay_builder = DelayBuilder::new();
	let (effect, delay_handle) = delay_builder.build();
	let delay_handle = builder.add_built_effect(effect);
	```
	*/
	pub fn add_built_effect(&mut self, effect: Box<dyn Effect>) {
		self.effects.push(effect);
	}

	/** Add an already-built effect and return the [`MainTrackBuilder`].

	`Box<dyn Effect>` values are created when calling `build` on an effect builder, which gives you
	an effect handle, as well as this boxed effect, which is the actual audio effect.

	This is a lower-level method than [`Self::with_effect`], and you should probably use it rather
	than this method, unless you have a reason to.

	# Examples

	```
	use kira::{
		track::MainTrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder, EffectBuilder},
	};

	let (filter_effect, filter_handle) = FilterBuilder::new().build();
	let (reverb_effect, reverb_handle) = ReverbBuilder::new().build();
	let mut builder = MainTrackBuilder::new()
		.with_built_effect(filter_effect)
		.with_built_effect(reverb_effect);
	```
	 */
	#[must_use = "This method consumes self and returns a modified MainTrackBuilder, so the return value should be used"]
	pub fn with_built_effect(mut self, effect: Box<dyn Effect>) -> Self {
		self.add_built_effect(effect);
		self
	}

	#[must_use]
	pub(crate) fn build(self) -> (MainTrack, MainTrackHandle) {
		let (set_volume_command_writer, set_volume_command_reader) = command_writer_and_reader();
		let (sounds, sound_controller) = ResourceStorage::new(self.sound_capacity);
		let track = MainTrack {
			volume: Parameter::new(self.volume, Decibels::IDENTITY),
			set_volume_command_reader,
			sounds,
			effects: self.effects,
		};
		let handle = MainTrackHandle {
			set_volume_command_writer,
			sound_controller,
		};
		(track, handle)
	}
}

impl Default for MainTrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}
