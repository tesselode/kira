use std::sync::atomic::Ordering;

use atomic_arena::Key;

use crate::{
	manager::command::{producer::CommandProducer, Command, SoundCommand},
	tween::{Tween, Value},
	CommandError, OutputDestination, StartTime, Volume,
};

use super::{wrapper::SoundWrapperShared, PlaybackState};

pub struct CommonSoundController {
	pub(crate) key: Key,
	pub(crate) command_producer: CommandProducer,
	pub(crate) shared: SoundWrapperShared,
}

impl CommonSoundController {
	/// Returns the current playback state of the sound.
	pub fn state(&self) -> PlaybackState {
		match self.shared.state.load(Ordering::SeqCst) {
			0 => PlaybackState::Playing,
			1 => PlaybackState::Pausing,
			2 => PlaybackState::Paused,
			3 => PlaybackState::Stopping,
			4 => PlaybackState::Stopped,
			_ => panic!("Invalid playback state"),
		}
	}

	/// Sets the volume of the sound.
	pub fn set_volume(
		&mut self,
		volume: impl Into<Value<Volume>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Sound(SoundCommand::SetVolume(
				self.key,
				volume.into(),
				tween,
			)))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	///	`0.5` is center, and `1.0` is hard right.
	pub fn set_panning(
		&mut self,
		panning: impl Into<Value<f64>>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Sound(SoundCommand::SetPanning(
				self.key,
				panning.into(),
				tween,
			)))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	pub fn pause(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Sound(SoundCommand::Pause(self.key, tween)))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	pub fn resume(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Sound(SoundCommand::Resume(self.key, tween)))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	pub fn stop(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Sound(SoundCommand::Stop(self.key, tween)))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommonSoundSettings {
	/// When the sound should start playing.
	pub start_time: StartTime,
	/// The volume of the sound.
	pub volume: Value<Volume>,
	/// The panning of the sound, where 0 is hard left
	/// and 1 is hard right.
	pub panning: Value<f64>,
	/// The destination that this sound should be routed to.
	pub output_destination: OutputDestination,
	/// An optional fade-in from silence.
	pub fade_in_tween: Option<Tween>,
}
