use std::{error::Error, fmt::Display, sync::Arc};

use ringbuf::Producer;

use crate::tween::Tween;

use super::{sound::Shared, Command, PlaybackState};

/// An error that occurs when trying to modify a static sound
/// whose command queue is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandQueueFull;

impl Display for CommandQueueFull {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot send a command to the sound because the command queue is full")
	}
}

impl Error for CommandQueueFull {}

/// Controls a static sound.
pub struct StaticSoundHandle {
	pub(super) command_producer: Producer<Command>,
	pub(super) shared: Arc<Shared>,
}

impl StaticSoundHandle {
	/// Returns the current playback state of the sound.
	pub fn state(&self) -> PlaybackState {
		self.shared.state()
	}

	/// Returns the current playback position of the sound (in seconds).
	pub fn position(&self) -> f64 {
		self.shared.position()
	}

	/// Sets the volume of the sound (as a factor of the original volume).
	///
	/// ## Example
	/// ```
	/// # use std::{error::Error, sync::Arc};
	/// #
	/// # use kira::{
	/// # 	manager::{backend::MockBackend, AudioManager},
	/// # 	sound::static_sound::StaticSoundData,
	/// # };
	/// #
	/// # let mut audio_manager = AudioManager::new(MockBackend::new(1), Default::default()).unwrap();
	/// # let mut sound = audio_manager.play(StaticSoundData {
	/// # 	sample_rate: 1,
	/// # 	frames: Arc::new(vec![]),
	/// # 	settings: Default::default(),
	/// # })?;
	/// #
	/// // set the volume to a fixed value
	/// sound.set_volume(0.75)?;
	///
	/// // set the volume to follow a parameter
	/// let parameter = audio_manager.add_parameter(0.5)?;
	/// sound.set_volume(&parameter)?;
	/// #
	/// # Result::<(), Box<dyn Error>>::Ok(())
	/// ```
	pub fn set_volume(&mut self, volume: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetVolume(volume))
			.map_err(|_| CommandQueueFull)
	}

	/// Sets the playback rate of the sound (as a factor of the
	/// original speed).
	///
	/// Changing the playback rate will change both the speed
	/// and pitch of the sound.
	///
	/// ## Example
	/// ```
	/// # use std::{error::Error, sync::Arc};
	/// #
	/// # use kira::{
	/// # 	manager::{backend::MockBackend, AudioManager},
	/// # 	sound::static_sound::StaticSoundData,
	/// # };
	/// #
	/// # let mut audio_manager = AudioManager::new(MockBackend::new(1), Default::default()).unwrap();
	/// # let mut sound = audio_manager.play(StaticSoundData {
	/// # 	sample_rate: 1,
	/// # 	frames: Arc::new(vec![]),
	/// # 	settings: Default::default(),
	/// # })?;
	/// #
	/// // set the playback rate to a fixed value
	/// sound.set_playback_rate(0.75)?;
	///
	/// // set the playback rate to follow a parameter
	/// let parameter = audio_manager.add_parameter(0.5)?;
	/// sound.set_playback_rate(&parameter)?;
	/// #
	/// # Result::<(), Box<dyn Error>>::Ok(())
	/// ```
	pub fn set_playback_rate(&mut self, playback_rate: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetPlaybackRate(playback_rate))
			.map_err(|_| CommandQueueFull)
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	/// `0.5` is center, and `1.0` is hard right.
	///
	/// ## Example
	/// ```
	/// # use std::{error::Error, sync::Arc};
	/// #
	/// # use kira::{
	/// # 	manager::{backend::MockBackend, AudioManager},
	/// # 	sound::static_sound::StaticSoundData,
	/// # };
	/// #
	/// # let mut audio_manager = AudioManager::new(MockBackend::new(1), Default::default()).unwrap();
	/// # let mut sound = audio_manager.play(StaticSoundData {
	/// # 	sample_rate: 1,
	/// # 	frames: Arc::new(vec![]),
	/// # 	settings: Default::default(),
	/// # })?;
	/// #
	/// // set the panning to a fixed value
	/// sound.set_panning(0.75)?;
	///
	/// // set the panning to follow a parameter
	/// let parameter = audio_manager.add_parameter(0.5)?;
	/// sound.set_panning(&parameter)?;
	/// #
	/// # Result::<(), Box<dyn Error>>::Ok(())
	/// ```
	pub fn set_panning(&mut self, panning: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SetPanning(panning))
			.map_err(|_| CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	pub fn pause(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Pause(tween))
			.map_err(|_| CommandQueueFull)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	pub fn resume(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Resume(tween))
			.map_err(|_| CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	pub fn stop(&mut self, tween: Tween) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::Stop(tween))
			.map_err(|_| CommandQueueFull)
	}

	/// Sets the playback position to the specified time in seconds.
	pub fn seek_to(&mut self, position: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SeekTo(position))
			.map_err(|_| CommandQueueFull)
	}

	/// Moves the playback position by the specified amount of time in seconds.
	pub fn seek_by(&mut self, amount: f64) -> Result<(), CommandQueueFull> {
		self.command_producer
			.push(Command::SeekBy(amount))
			.map_err(|_| CommandQueueFull)
	}
}
