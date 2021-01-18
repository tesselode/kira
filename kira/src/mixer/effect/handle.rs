//! An interface for controlling effects.

use flume::Sender;
use thiserror::Error;

use crate::{
	command::{Command, MixerCommand},
	mixer::TrackIndex,
};

use super::{EffectId, EffectSettings};

/// Something that can go wrong when using an [`EffectHandle`] to
/// control an effect.
#[derive(Debug, Error)]
pub enum EffectHandleError {
	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Allows you to control an effect.
pub struct EffectHandle {
	id: EffectId,
	track_index: TrackIndex,
	enabled: bool,
	command_sender: Sender<Command>,
}

impl EffectHandle {
	pub(crate) fn new(
		track_index: TrackIndex,
		settings: &EffectSettings,
		command_sender: Sender<Command>,
	) -> Self {
		Self {
			id: settings.id,
			enabled: settings.enabled,
			track_index,
			command_sender,
		}
	}

	/// Returns the ID of the effect.
	pub fn id(&self) -> EffectId {
		self.id
	}

	/// Returns the track that contains this effect.
	pub fn track_index(&self) -> TrackIndex {
		self.track_index
	}

	/// Returns whether the effect is currently enabled.
	pub fn enabled(&self) -> bool {
		self.enabled
	}

	/// Sets whether the effect is currently enabled.
	pub fn set_enabled(&mut self, enabled: bool) -> Result<(), EffectHandleError> {
		self.enabled = enabled;
		self.command_sender
			.send(MixerCommand::SetEffectEnabled(self.track_index, self.id, enabled).into())
			.map_err(|_| EffectHandleError::BackendDisconnected)
	}
}
