use std::sync::Arc;

use crate::{
	Decibels, Panning, PlaybackRate, StartTime, Tween,
	command::handle_param_setters,
	sound::{IntoOptionalRegion, PlaybackState},
};

use super::{CommandWriters, sound::Shared};

/// Controls a static sound.
#[derive(Debug)]
pub struct StaticSoundHandle {
	pub(super) command_writers: CommandWriters,
	pub(super) shared: Arc<Shared>,
}

impl StaticSoundHandle {
	/// Returns the current playback state of the sound.
	#[must_use]
	pub fn state(&self) -> PlaybackState {
		self.shared.state()
	}

	/// Returns the current playback position of the sound (in seconds).
	#[must_use]
	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	handle_param_setters! {
		/**
		Sets the volume of the sound.

		# Examples

		Set the volume of the sound immediately:

		```no_run
		# use kira::{
		# 	AudioManager, AudioManagerSettings, DefaultBackend,
		# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
		# };
		# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		use kira::Tween;

		sound.set_volume(-6.0, Tween::default());
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```

		Smoothly transition the volume to a target volume:

		```no_run
		# use kira::{
		# 	AudioManager, AudioManagerSettings, DefaultBackend,
		# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
		# };
		# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		use kira::Tween;
		use std::time::Duration;

		sound.set_volume(-6.0, Tween {
			duration: Duration::from_secs(3),
			..Default::default()
		});
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```

		Link the volume to a modulator, smoothly transitioning from the current value:

		```no_run
		use kira::{
			AudioManager, AudioManagerSettings, DefaultBackend,
			sound::static_sound::{StaticSoundData, StaticSoundSettings},
			modulator::tweener::TweenerBuilder,
			Value, Tween, Mapping, Easing,
			Decibels,
		};
		use std::time::Duration;

		let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		let tweener = manager.add_modulator(TweenerBuilder {
			initial_value: 0.5,
		})?;
		let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		sound.set_volume(Value::FromModulator {
			id: tweener.id(),
			mapping: Mapping {
				input_range: (0.0, 1.0),
				output_range: (Decibels::SILENCE, Decibels::IDENTITY),
				easing: Easing::Linear,
			}
		}, Tween {
			duration: Duration::from_secs(3),
			..Default::default()
		});
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```
		*/
		volume: Decibels,

		/**
		Sets the playback rate of the sound.

		Changing the playback rate will change both the speed
		and pitch of the sound.

		# Examples

		Set the playback rate of the sound immediately:

		```no_run
		# use kira::{
		# 	AudioManager, AudioManagerSettings, DefaultBackend,
		# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
		# };
		# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		use kira::Tween;

		sound.set_playback_rate(0.5, Tween::default());
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```

		Smoothly transition the playback rate to a target value in semitones:

		```no_run
		# use kira::{
		# 	AudioManager, AudioManagerSettings, DefaultBackend,
		# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
		# };
		# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		use kira::{Tween, Semitones};
		use std::time::Duration;

		sound.set_playback_rate(Semitones(-2.0), Tween {
			duration: Duration::from_secs(3),
			..Default::default()
		});
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```

		Link the playback rate to a modulator, smoothly transitioning from the current value:

		```no_run
		use kira::{
			AudioManager, AudioManagerSettings, DefaultBackend,
			sound::static_sound::{StaticSoundData, StaticSoundSettings},
			modulator::tweener::TweenerBuilder,
			Value, Easing, Mapping, Tween,
			PlaybackRate,
		};
		use std::time::Duration;

		let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		let tweener = manager.add_modulator(TweenerBuilder {
			initial_value: 0.5,
		})?;
		let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		sound.set_playback_rate(Value::FromModulator {
			id: tweener.id(),
			mapping: Mapping {
				input_range: (0.0, 1.0),
				output_range: (PlaybackRate(0.0), PlaybackRate(1.0)),
				easing: Easing::Linear,
			},
		}, Tween {
			duration: Duration::from_secs(3),
			..Default::default()
		});
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```
		*/
		playback_rate: PlaybackRate,

		/**
		Sets the panning of the sound, where `-1.0` is hard left,
		`0.0` is center, and `1.0` is hard right.

		# Examples

		Smoothly transition the panning to a target value:

		```no_run
		# use kira::{
		# 	AudioManager, AudioManagerSettings, DefaultBackend,
		# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
		# };
		# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		use kira::Tween;
		use std::time::Duration;

		sound.set_panning(-0.5, Tween {
			duration: Duration::from_secs(3),
			..Default::default()
		});
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```

		Link the panning to a modulator, smoothly transitioning from the current value:

		```no_run
		use kira::{
			AudioManager, AudioManagerSettings, DefaultBackend,
			sound::static_sound::{StaticSoundData, StaticSoundSettings},
			modulator::tweener::TweenerBuilder,
			Value, Easing, Mapping, Tween,
			Panning,
		};
		use std::time::Duration;

		let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
		let tweener = manager.add_modulator(TweenerBuilder {
			initial_value: -0.5,
		})?;
		let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
		sound.set_panning(Value::FromModulator {
			id: tweener.id(),
			mapping: Mapping {
				input_range: (-1.0, 1.0),
				output_range: (Panning::LEFT, Panning::RIGHT),
				easing: Easing::Linear,
			},
		}, Tween {
			duration: Duration::from_secs(3),
			..Default::default()
		});
		# Result::<(), Box<dyn std::error::Error>>::Ok(())
		```
		*/
		panning: Panning,
	}

	/**
	Sets the portion of the sound that will play in a loop.

	# Examples

	Set the sound to loop the portion from 3 seconds in to the end:

	```no_run
	# use kira::{
	# 	AudioManager, AudioManagerSettings, DefaultBackend,
	# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
	sound.set_loop_region(3.0..);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the sound to loop the portion from 2 to 4 seconds:

	```no_run
	# use kira::{
	# 	AudioManager, AudioManagerSettings, DefaultBackend,
	# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
	sound.set_loop_region(2.0..4.0);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set a sound that was previously looping to stop looping:

	```no_run
	# use kira::{
	# 	AudioManager, AudioManagerSettings, DefaultBackend,
	# 	sound::static_sound::{StaticSoundData, StaticSoundSettings},
	# };
	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# let mut sound = manager.play(StaticSoundData::from_file("sound.ogg")?)?;
	sound.set_loop_region(None);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn set_loop_region(&mut self, loop_region: impl IntoOptionalRegion) {
		self.command_writers
			.set_loop_region
			.write(loop_region.into_optional_region())
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	pub fn pause(&mut self, tween: Tween) {
		self.command_writers.pause.write(tween)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	pub fn resume(&mut self, tween: Tween) {
		self.resume_at(StartTime::Immediate, tween)
	}

	/// Resumes playback at the given start time and fades in
	/// the sound from silence with the given tween.
	pub fn resume_at(&mut self, start_time: StartTime, tween: Tween) {
		self.command_writers.resume.write((start_time, tween))
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	pub fn stop(&mut self, tween: Tween) {
		self.command_writers.stop.write(tween)
	}

	/// Sets the playback position to the specified time in seconds.
	pub fn seek_to(&mut self, position: f64) {
		self.command_writers.seek_to.write(position)
	}

	/// Moves the playback position by the specified amount of time in seconds.
	pub fn seek_by(&mut self, amount: f64) {
		self.command_writers.seek_by.write(amount)
	}
}
