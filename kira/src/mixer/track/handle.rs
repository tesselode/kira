//! An interface for controlling mixer tracks.

use flume::Sender;
use thiserror::Error;

use crate::{
	command::{Command, MixerCommand},
	mixer::effect::{handle::EffectHandle, Effect, EffectId, EffectSettings},
};

use super::TrackIndex;

/// Something that can go wrong when using a [`TrackHandle`] to
/// control a mixer track.
#[derive(Debug, Error)]
pub enum TrackHandleError {
	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

#[derive(Debug, Clone)]
/// Allows you to control a mixer sound.
pub struct TrackHandle {
	index: TrackIndex,
	command_sender: Sender<Command>,
}

impl TrackHandle {
	pub(crate) fn new(index: TrackIndex, command_sender: Sender<Command>) -> Self {
		Self {
			index,
			command_sender,
		}
	}

	/// Gets the track that this handle controls.
	pub fn index(&self) -> TrackIndex {
		self.index
	}

	/// Adds an effect to the track.
	pub fn add_effect(
		&mut self,
		effect: impl Effect + 'static,
		settings: EffectSettings,
	) -> Result<EffectHandle, TrackHandleError> {
		let handle = EffectHandle::new(self.index, &settings, self.command_sender.clone());
		self.command_sender
			.send(MixerCommand::AddEffect(self.index, Box::new(effect), settings).into())
			.map_err(|_| TrackHandleError::BackendDisconnected)?;
		Ok(handle)
	}

	/// Removes an effect from the track.
	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), TrackHandleError> {
		self.command_sender
			.send(MixerCommand::RemoveEffect(self.index, id.into()).into())
			.map_err(|_| TrackHandleError::BackendDisconnected)
	}
}
