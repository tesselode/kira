use std::sync::Arc;

use crate::{
	command::command_writer_and_reader,
	effect::EffectBuilder,
	frame::Frame,
	tween::{Parameter, Value},
	Decibels,
};

use super::{Effect, SendTrack, SendTrackHandle, SendTrackId, TrackShared};

/// Configures a mixer track.
pub struct SendTrackBuilder {
	/// The volume of the send track.
	pub(crate) volume: Value<Decibels>,
	/// The effects that should be applied to the input audio
	/// for this track.
	pub(crate) effects: Vec<Box<dyn Effect>>,
}

impl SendTrackBuilder {
	/// Creates a new [`SendTrackBuilder`] with the default settings.
	#[must_use]
	pub fn new() -> Self {
		Self {
			volume: Value::Fixed(Decibels::IDENTITY),
			effects: vec![],
		}
	}

	/**
	Sets the volume of the send track.

	# Examples

	Set the volume to a fixed decibel value:

	```
	# use kira::track::SendTrackBuilder;
	let builder = SendTrackBuilder::new().volume(-6.0);
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		track::SendTrackBuilder,
		tween::{Easing, Value, Mapping},
		Decibels,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let builder = SendTrackBuilder::new().volume(Value::FromModulator {
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
	#[must_use = "This method consumes self and returns a modified SendTrackBuilder, so the return value should be used"]
	pub fn volume(self, volume: impl Into<Value<Decibels>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/**
	Adds an effect to the send track.

	# Examples

	```
	use kira::{track::SendTrackBuilder, effect::delay::DelayBuilder};

	let mut builder = SendTrackBuilder::new();
	let delay_handle = builder.add_effect(DelayBuilder::new());
	```
	*/
	pub fn add_effect<B: EffectBuilder>(&mut self, builder: B) -> B::Handle {
		let (effect, handle) = builder.build();
		self.effects.push(effect);
		handle
	}

	/**
	Adds an effect to the send track and returns the [`SendTrackBuilder`].

	If you need to modify the effect later, use [`add_effect`](Self::add_effect),
	which returns the effect handle.

	# Examples

	```
	use kira::{
		track::SendTrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder},
	};

	let mut builder = SendTrackBuilder::new()
		.with_effect(FilterBuilder::new())
		.with_effect(ReverbBuilder::new());
	```
	*/
	#[must_use = "This method consumes self and returns a modified SendTrackBuilder, so the return value should be used"]
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
	use kira::track::SendTrackBuilder;
	use kira::effect::{EffectBuilder, delay::DelayBuilder};

	let mut builder = SendTrackBuilder::new();
	let delay_builder = DelayBuilder::new();
	let (effect, delay_handle) = delay_builder.build();
	let delay_handle = builder.add_built_effect(effect);
	```
	*/
	pub fn add_built_effect(&mut self, effect: Box<dyn Effect>) {
		self.effects.push(effect);
	}

	/** Add an already-built effect and return the [`SendTrackBuilder`].

	 `Box<dyn Effect>` values are created when calling `build` on an effect builder, which gives you
	 an effect handle, as well as this boxed effect, which is the actual audio effect.

	 This is a lower-level method than [`Self::with_effect`], and you should probably use it rather
	 than this method, unless you have a reason to.

	# Examples

	```
	use kira::{
		track::SendTrackBuilder,
		effect::{filter::FilterBuilder, reverb::ReverbBuilder, EffectBuilder},
	};

	let (filter_effect, filter_handle) = FilterBuilder::new().build();
	let (reverb_effect, reverb_handle) = ReverbBuilder::new().build();
	let mut builder = SendTrackBuilder::new()
		.with_built_effect(filter_effect)
		.with_built_effect(reverb_effect);
	```
	 */
	#[must_use = "This method consumes self and returns a modified SendTrackBuilder, so the return value should be used"]
	pub fn with_built_effect(mut self, effect: Box<dyn Effect>) -> Self {
		self.add_built_effect(effect);
		self
	}

	#[must_use]
	pub(crate) fn build(self, id: SendTrackId) -> (SendTrack, SendTrackHandle) {
		let (set_volume_command_writer, set_volume_command_reader) = command_writer_and_reader();
		let shared = Arc::new(TrackShared::new());
		let track = SendTrack {
			shared: shared.clone(),
			volume: Parameter::new(self.volume, Decibels::IDENTITY),
			set_volume_command_reader,
			effects: self.effects,
			input: Frame::ZERO,
		};
		let handle = SendTrackHandle {
			id,
			shared,
			set_volume_command_writer,
		};
		(track, handle)
	}
}

impl Default for SendTrackBuilder {
	fn default() -> Self {
		Self::new()
	}
}
