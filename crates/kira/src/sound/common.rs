use std::sync::atomic::Ordering;

use atomic_arena::Key;

use crate::{
	command::ValueChangeCommand,
	manager::command::producer::CommandProducer,
	tween::{Tween, Value},
	OutputDestination, StartTime, Volume,
};

use super::{
	wrapper::{
		CommandWriters, PlaybackStateChangeCommand, PlaybackStateChangeCommandKind,
		SoundWrapperShared,
	},
	PlaybackState,
};

pub struct CommonSoundController {
	pub(crate) key: Key,
	pub(crate) command_producer: CommandProducer,
	pub(crate) shared: SoundWrapperShared,
	pub(crate) command_writers: CommandWriters,
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
	pub fn set_volume(&mut self, volume: impl Into<Value<Volume>>, tween: Tween) {
		self.command_writers
			.volume_change
			.write(ValueChangeCommand {
				target: volume.into(),
				tween,
			});
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	///	`0.5` is center, and `1.0` is hard right.
	pub fn set_panning(&mut self, panning: impl Into<Value<f64>>, tween: Tween) {
		self.command_writers
			.panning_change
			.write(ValueChangeCommand {
				target: panning.into(),
				tween,
			});
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	pub fn pause(&mut self, tween: Tween) {
		self.command_writers
			.playback_state_change
			.write(PlaybackStateChangeCommand {
				kind: PlaybackStateChangeCommandKind::Pause,
				fade_tween: tween,
			})
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	pub fn resume(&mut self, tween: Tween) {
		self.command_writers
			.playback_state_change
			.write(PlaybackStateChangeCommand {
				kind: PlaybackStateChangeCommandKind::Resume,
				fade_tween: tween,
			})
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	pub fn stop(&mut self, tween: Tween) {
		self.command_writers
			.playback_state_change
			.write(PlaybackStateChangeCommand {
				kind: PlaybackStateChangeCommandKind::Stop,
				fade_tween: tween,
			})
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
