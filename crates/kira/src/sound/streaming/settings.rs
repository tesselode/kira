use crate::{
	sound::{IntoOptionalRegion, PlaybackPosition, PlaybackRate, Region},
	tween::{Tween, Value},
	OutputDestination, StartTime, Volume,
};

/// Settings for a streaming sound.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct StreamingSoundSettings {
	/// When the sound should start playing.
	pub start_time: StartTime,
	/// Where in the sound playback should start.
	pub start_position: PlaybackPosition,
	/// The portion of the sound that should be looped.
	pub loop_region: Option<Region>,
	/// The volume of the sound.
	pub volume: Value<Volume>,
	/// The playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and the pitch of the sound.
	pub playback_rate: Value<PlaybackRate>,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value<f64>,
	/// The destination that this sound should be routed to.
	pub output_destination: OutputDestination,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}

impl StreamingSoundSettings {
	/// Creates a new [`StreamingSoundSettings`] with the default settings.
	pub fn new() -> Self {
		Self {
			start_time: StartTime::Immediate,
			start_position: PlaybackPosition::Samples(0),
			loop_region: None,
			volume: Value::Fixed(Volume::Amplitude(1.0)),
			playback_rate: Value::Fixed(PlaybackRate::Factor(1.0)),
			panning: Value::Fixed(0.5),
			output_destination: OutputDestination::default(),
			fade_in_tween: None,
		}
	}

	/**
	Sets when the sound should start playing.

	# Examples

	Configuring a sound to start 4 ticks after a clock's current time:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		sound::streaming::{StreamingSoundData, StreamingSoundSettings},
		clock::ClockSpeed,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let clock_handle = manager.add_clock(ClockSpeed::TicksPerMinute(120.0))?;
	let settings = StreamingSoundSettings::new().start_time(clock_handle.time() + 4);
	let sound = StreamingSoundData::from_file("sound.ogg", settings);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn start_time(self, start_time: impl Into<StartTime>) -> Self {
		Self {
			start_time: start_time.into(),
			..self
		}
	}

	/**
	Sets where in the sound playback should start.

	# Examples

	Configure a sound to start 1 second into the audio data:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().start_position(1.0);
	```

	Configure a sound to start 50 samples into the audio data:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	# use kira::sound::PlaybackPosition;
	let settings = StreamingSoundSettings::new().start_position(PlaybackPosition::Samples(50));
	```
	*/
	pub fn start_position(self, start_position: impl Into<PlaybackPosition>) -> Self {
		Self {
			start_position: start_position.into(),
			..self
		}
	}

	/**
	Sets the portion of the sound that should be looped.

	# Examples

	Configure a sound to loop the portion from 3 seconds in to the end:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().loop_region(3.0..);
	```

	Configure a sound to loop the portion from 2 to 4 seconds:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().loop_region(2.0..4.0);
	```
	*/
	pub fn loop_region(self, loop_region: impl IntoOptionalRegion) -> Self {
		Self {
			loop_region: loop_region.into_optional_region(),
			..self
		}
	}

	/**
	Sets the volume of the sound.

	# Examples

	Set the volume as a factor:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().volume(0.5);
	```

	Set the volume as a gain in decibels:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().volume(kira::Volume::Decibels(-6.0));
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::streaming::{StreamingSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let settings = StreamingSoundSettings::new().volume(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn volume(self, volume: impl Into<Value<Volume>>) -> Self {
		Self {
			volume: volume.into(),
			..self
		}
	}

	/**
	Sets the playback rate of the sound.

	Changing the playback rate will change both the speed
	and the pitch of the sound.

	# Examples

	Set the playback rate as a factor:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().playback_rate(0.5);
	```

	Set the playback rate as a change in semitones:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	use kira::sound::PlaybackRate;
	let settings = StreamingSoundSettings::new().playback_rate(PlaybackRate::Semitones(-2.0));
	```

	Link the playback rate to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::streaming::{StreamingSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let settings = StreamingSoundSettings::new().playback_rate(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn playback_rate(self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		Self {
			playback_rate: playback_rate.into(),
			..self
		}
	}

	/**
	Sets the panning of the sound, where 0 is hard left
	and 1 is hard right.

	# Examples

	Set the panning to a streaming value:

	```
	# use kira::sound::streaming::StreamingSoundSettings;
	let settings = StreamingSoundSettings::new().panning(0.25);
	```

	Link the panning to a modulator:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		modulator::tweener::TweenerBuilder,
		sound::streaming::{StreamingSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.25,
	})?;
	let settings = StreamingSoundSettings::new().panning(&tweener);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn panning(self, panning: impl Into<Value<f64>>) -> Self {
		Self {
			panning: panning.into(),
			..self
		}
	}

	/**
	Sets the destination that this sound should be routed to.

	# Examples

	Set the output destination of a sound to a mixer track:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		track::TrackBuilder,
		sound::streaming::{StreamingSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let sub_track = manager.add_sub_track(TrackBuilder::new())?;
	let settings = StreamingSoundSettings::new().output_destination(&sub_track);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the output destination of a sound to an emitter in a spatial scene:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		spatial::{scene::SpatialSceneSettings, emitter::EmitterSettings},
		sound::streaming::{StreamingSoundSettings},
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let mut scene = manager.add_spatial_scene(SpatialSceneSettings::default())?;
	let emitter = scene.add_emitter(mint::Vector3 {
		x: 0.0,
		y: 0.0,
		z: 0.0,
	}, EmitterSettings::default())?;
	let settings = StreamingSoundSettings::new().output_destination(&emitter);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn output_destination(self, output_destination: impl Into<OutputDestination>) -> Self {
		Self {
			output_destination: output_destination.into(),
			..self
		}
	}

	/// Sets the tween used to fade in the instance from silence.
	pub fn fade_in_tween(self, fade_in_tween: impl Into<Option<Tween>>) -> Self {
		Self {
			fade_in_tween: fade_in_tween.into(),
			..self
		}
	}
}

impl Default for StreamingSoundSettings {
	fn default() -> Self {
		Self::new()
	}
}
