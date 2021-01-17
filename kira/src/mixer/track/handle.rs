use flume::Sender;
use thiserror::Error;

use crate::{
	audio_stream::{AudioStream, AudioStreamId},
	command::{Command, MixerCommand, StreamCommand},
	mixer::effect::{handle::EffectHandle, Effect, EffectId, EffectSettings},
};

use super::TrackIndex;

#[derive(Debug, Error)]
pub enum TrackHandleError {
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

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

	pub fn index(&self) -> TrackIndex {
		self.index
	}

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

	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), TrackHandleError> {
		self.command_sender
			.send(MixerCommand::RemoveEffect(self.index, id.into()).into())
			.map_err(|_| TrackHandleError::BackendDisconnected)
	}
}
