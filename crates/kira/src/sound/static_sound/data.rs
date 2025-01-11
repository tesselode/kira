#[cfg(feature = "symphonia")]
mod from_file;

#[cfg(test)]
mod test;

use std::{
	fmt::{Debug, Formatter},
	sync::Arc,
	time::Duration,
};

use crate::{
	frame::Frame,
	sound::{EndPosition, IntoOptionalRegion, PlaybackPosition, Region, Sound, SoundData},
	Tween,
	Decibels, Panning, PlaybackRate, StartTime, Value,
};

use super::{
	command_writers_and_readers, handle::StaticSoundHandle, sound::StaticSound, StaticSoundSettings,
};

/// A piece of audio loaded into memory all at once.
///
/// These can be cheaply cloned, as the audio data is shared
/// among all clones.
#[derive(Clone, PartialEq)]
pub struct StaticSoundData {
	/// The sample rate of the audio (in Hz).
	pub sample_rate: u32,
	/// The raw samples that make up the audio.
	pub frames: Arc<[Frame]>,
	/// Settings for the sound.
	pub settings: StaticSoundSettings,
	/**
	The portion of the sound this [`StaticSoundData`] represents.

	Note that the [`StaticSoundData`] holds the entire piece of audio
	it was originally given regardless of the value of `slice`, but
	[`StaticSoundData::num_frames`], [`StaticSoundData::duration`],
	and [`StaticSoundData::frame_at_index`] will all behave as if
	this [`StaticSoundData`] only contained the specified portion of
	audio.
	*/
	pub slice: Option<(usize, usize)>,
}

impl StaticSoundData {
	/**
	Sets when the sound should start playing.

	This returns a cheap clone of the [`StaticSoundData`] with the modified start time.

	# Examples

	Configuring a sound to start 4 ticks after a clock's current time:

	```no_run
	use kira::{
		AudioManager, AudioManagerSettings, DefaultBackend,
		sound::static_sound::{StaticSoundData, StaticSoundSettings},
		clock::ClockSpeed,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let clock_handle = manager.add_clock(ClockSpeed::TicksPerMinute(120.0))?;
	let sound = StaticSoundData::from_file("sound.ogg")?
		.start_time(clock_handle.time() + 4);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn start_time(&self, start_time: impl Into<StartTime>) -> Self {
		let mut new = self.clone();
		new.settings.start_time = start_time.into();
		new
	}

	/// Sets where in the sound playback should start.
	///
	/// This returns a cheap clone of the [`StaticSoundData`] with the modified start position.
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn start_position(&self, start_position: impl Into<PlaybackPosition>) -> Self {
		let mut new = self.clone();
		new.settings.start_position = start_position.into();
		new
	}

	/// Sets whether the sound should be played in reverse.
	///
	/// This returns a cheap clone of the [`StaticSoundData`] with the modified setting.
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn reverse(&self, reverse: bool) -> Self {
		let mut new = self.clone();
		new.settings.reverse = reverse;
		new
	}

	/**
	Sets the portion of the sound that should be looped.

	This returns a cheap clone of the [`StaticSoundData`] with the modified loop region.

	# Examples

	Configure a sound to loop the portion from 3 seconds in to the end:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().loop_region(3.0..);
	```

	Configure a sound to loop the portion from 2 to 4 seconds:

	```
	# use kira::sound::static_sound::StaticSoundSettings;
	let settings = StaticSoundSettings::new().loop_region(2.0..4.0);
	```
	*/
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn loop_region(&self, loop_region: impl IntoOptionalRegion) -> Self {
		let mut new = self.clone();
		new.settings.loop_region = loop_region.into_optional_region();
		new
	}

	/**
	Sets the volume of the sound.

	# Examples

	Set the volume to a fixed value:

	```no_run
	# use kira::sound::static_sound::StaticSoundData;
	let sound = StaticSoundData::from_file("sound.ogg")?.volume(-6.0);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the volume to a modulator:

	```no_run
	use kira::{
		AudioManager, AudioManagerSettings, DefaultBackend,
		modulator::tweener::TweenerBuilder,
		sound::static_sound::StaticSoundData,
		Value, Mapping, Easing,
		Decibels,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let sound = StaticSoundData::from_file("sound.ogg")?.volume(Value::FromModulator {
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
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn volume(&self, volume: impl Into<Value<Decibels>>) -> Self {
		let mut new = self.clone();
		new.settings.volume = volume.into();
		new
	}

	/**
	Sets the playback rate of the sound.

	Changing the playback rate will change both the speed
	and the pitch of the sound.

	# Examples

	Set the playback rate as a factor:

	```no_run
	# use kira::sound::static_sound::StaticSoundData;
	let sound = StaticSoundData::from_file("sound.ogg")?.playback_rate(0.5);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Set the playback rate as a change in semitones:

	```no_run
	# use kira::sound::static_sound::StaticSoundData;
	use kira::Semitones;
	let sound = StaticSoundData::from_file("sound.ogg")?.playback_rate(Semitones(-2.0));
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the playback rate to a modulator:

	```no_run
	use kira::{
		AudioManager, AudioManagerSettings, DefaultBackend,
		modulator::tweener::TweenerBuilder,
		sound::static_sound::StaticSoundData,
		Value, Easing, Mapping,
		PlaybackRate,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: 0.5,
	})?;
	let sound = StaticSoundData::from_file("sound.ogg")?.playback_rate(Value::FromModulator {
		id: tweener.id(),
		mapping: Mapping {
			input_range: (0.0, 1.0),
			output_range: (PlaybackRate(0.0), PlaybackRate(1.0)),
			easing: Easing::Linear,
		},
	});
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn playback_rate(&self, playback_rate: impl Into<Value<PlaybackRate>>) -> Self {
		let mut new = self.clone();
		new.settings.playback_rate = playback_rate.into();
		new
	}

	/**
	Sets the panning of the sound, where 0 is hard left
	and 1 is hard right.

	# Examples

	Set the panning to a fixed value:

	``` no_run
	# use kira::sound::static_sound::StaticSoundData;
	let sound = StaticSoundData::from_file("sound.ogg")?.panning(-0.5);
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Link the panning to a modulator:

	```no_run
	use kira::{
		AudioManager, AudioManagerSettings, DefaultBackend,
		modulator::tweener::TweenerBuilder,
		sound::static_sound::StaticSoundData,
		Value, Easing, Mapping,
		Panning,
	};

	let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let tweener = manager.add_modulator(TweenerBuilder {
		initial_value: -0.5,
	})?;
	let sound = StaticSoundData::from_file("sound.ogg")?.panning(Value::FromModulator {
		id: tweener.id(),
		mapping: Mapping {
			input_range: (-1.0, 1.0),
			output_range: (Panning::LEFT, Panning::RIGHT),
			easing: Easing::Linear,
		},
	});
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn panning(&self, panning: impl Into<Value<Panning>>) -> Self {
		let mut new = self.clone();
		new.settings.panning = panning.into();
		new
	}

	/// Sets the tween used to fade in the sound from silence.
	///
	/// This returns a cheap clone of the [`StaticSoundData`] with the modified fade in tween.
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn fade_in_tween(&self, fade_in_tween: impl Into<Option<Tween>>) -> Self {
		let mut new = self.clone();
		new.settings.fade_in_tween = fade_in_tween.into();
		new
	}

	/// Returns a cheap clone of the `StaticSoundData` with the specified settings.
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn with_settings(&self, settings: StaticSoundSettings) -> Self {
		Self {
			settings,
			..self.clone()
		}
	}

	/// Returns the number of frames in the [`StaticSoundData`].
	///
	/// If [`StaticSoundData::slice`] is `Some`, this will be the number
	/// of frames in the slice.
	#[must_use]
	pub fn num_frames(&self) -> usize {
		num_frames(&self.frames, self.slice)
	}

	/// Returns the duration of the audio.
	///
	/// If [`StaticSoundData::slice`] is `Some`, this will be the duration
	/// of the slice.
	#[must_use]
	pub fn duration(&self) -> Duration {
		Duration::from_secs_f64(self.num_frames() as f64 / self.sample_rate as f64)
	}

	/// Returns the total duration of the audio, regardless of its slice.
	#[must_use]
	pub fn unsliced_duration(&self) -> Duration {
		Duration::from_secs_f64(self.frames.len() as f64 / self.sample_rate as f64)
	}

	/// Returns the nth [`Frame`] of audio in the [`StaticSoundData`].
	///
	/// If [`StaticSoundData::slice`] is `Some`, this will behave as if the [`StaticSoundData`]
	/// only contained that portion of the audio.
	#[must_use]
	pub fn frame_at_index(&self, index: usize) -> Option<Frame> {
		frame_at_index(index, &self.frames, self.slice)
	}

	/**
	Sets the portion of the audio this [`StaticSoundData`] represents.

	This returns a cheap clone of the [`StaticSoundData`] with the modified slice.

	Note that the [`StaticSoundData`] holds the entire piece of audio it was originally
	given regardless of the value of `slice`, but [`StaticSoundData::num_frames`],
	[`StaticSoundData::duration`], and [`StaticSoundData::frame_at_index`] will all behave
	as if this [`StaticSoundData`] only contained the specified portion of audio.

	# Example

	```
	use kira::{
		sound::static_sound::{StaticSoundData, StaticSoundSettings},
		Frame,
	};
	let sound = StaticSoundData {
		sample_rate: 1,
		frames: (0..10).map(|i| Frame::from_mono(i as f32)).collect(),
		settings: StaticSoundSettings::default(),
		slice: None,
	};
	let sliced = sound.slice(3.0..6.0);
	assert_eq!(sliced.num_frames(), 3);
	assert_eq!(sliced.frame_at_index(0), Some(Frame::from_mono(3.0)));
	assert_eq!(sliced.frame_at_index(1), Some(Frame::from_mono(4.0)));
	```
	*/
	#[must_use = "This method returns a modified StaticSoundData and does not mutate the original value"]
	pub fn slice(&self, region: impl IntoOptionalRegion) -> Self {
		let mut new = self.clone();
		new.slice = region.into_optional_region().map(|Region { start, end }| {
			let start = start.into_samples(self.sample_rate);
			let end = match end {
				EndPosition::EndOfAudio => self.frames.len(),
				EndPosition::Custom(end) => end.into_samples(self.sample_rate),
			};
			(start, end)
		});
		new
	}

	pub(super) fn split(self) -> (StaticSound, StaticSoundHandle) {
		let (command_writers, command_readers) = command_writers_and_readers();
		let sound = StaticSound::new(self, command_readers);
		let shared = sound.shared();
		(
			sound,
			StaticSoundHandle {
				command_writers,
				shared,
			},
		)
	}
}

impl SoundData for StaticSoundData {
	type Error = ();

	type Handle = StaticSoundHandle;

	#[allow(clippy::type_complexity)]
	fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
		let (sound, handle) = self.split();
		Ok((Box::new(sound), handle))
	}
}

impl Debug for StaticSoundData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("StaticSoundData")
			.field("sample_rate", &self.sample_rate)
			.field(
				"frames",
				&FramesDebug {
					len: self.frames.len(),
				},
			)
			.field("settings", &self.settings)
			.finish()
	}
}

struct FramesDebug {
	len: usize,
}

impl Debug for FramesDebug {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("[{} frames]", self.len))
	}
}

pub(crate) fn num_frames(frames: &[Frame], slice: Option<(usize, usize)>) -> usize {
	if let Some((start, end)) = slice {
		end - start
	} else {
		frames.len()
	}
}

pub(crate) fn frame_at_index(
	index: usize,
	frames: &[Frame],
	slice: Option<(usize, usize)>,
) -> Option<Frame> {
	if index >= num_frames(frames, slice) {
		return None;
	}
	let start = slice.map(|(start, _)| start).unwrap_or_default();
	Some(frames[index + start])
}
