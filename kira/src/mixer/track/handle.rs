//! An interface for controlling mixer tracks.

use basedrop::Owned;
use flume::Sender;
use indexmap::IndexSet;
use thiserror::Error;

use crate::{
	command::{Command, MixerCommand},
	mixer::effect::{handle::EffectHandle, Effect, EffectId, EffectSettings},
};

use super::{TrackIndex, TrackSettings};

/// Something that can go wrong when using a [`TrackHandle`] to
/// control a mixer track.
#[derive(Debug, Error)]
pub enum TrackHandleError {
	/// The maximum effect limit for this track has been reached.
	#[error(
		"Cannot add an effect because the max number of effects for this track has been reached"
	)]
	EffectLimitReached,
	/// No effect with the specified ID exists on this track.
	#[error("No effect with the specified ID exists on this track")]
	NoEffectWithId(EffectId),
	/// The audio thread has finished and can no longer receive commands.
	#[error("The backend cannot receive commands because it no longer exists")]
	BackendDisconnected,
}

/// Allows you to control a mixer sound.
pub struct TrackHandle {
	index: TrackIndex,
	command_sender: Sender<Command>,
	active_effect_ids: IndexSet<EffectId>,
	resource_collector_handle: basedrop::Handle,
}

impl TrackHandle {
	pub(crate) fn new(
		index: TrackIndex,
		settings: &TrackSettings,
		command_sender: Sender<Command>,
		resource_collector_handle: basedrop::Handle,
	) -> Self {
		Self {
			index,
			command_sender,
			active_effect_ids: IndexSet::with_capacity(settings.num_effects),
			resource_collector_handle,
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
		if self.active_effect_ids.len() >= self.active_effect_ids.capacity() {
			return Err(TrackHandleError::EffectLimitReached);
		}
		let id = settings.id;
		let handle = EffectHandle::new(self.index, &settings, self.command_sender.clone());
		self.command_sender
			.send(
				MixerCommand::AddEffect(
					self.index,
					Owned::new(&self.resource_collector_handle, Box::new(effect)),
					settings,
				)
				.into(),
			)
			.map_err(|_| TrackHandleError::BackendDisconnected)?;
		self.active_effect_ids.insert(id);
		Ok(handle)
	}

	/// Removes an effect from the track.
	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), TrackHandleError> {
		let id = id.into();
		if !self.active_effect_ids.remove(&id) {
			return Err(TrackHandleError::NoEffectWithId(id));
		}
		self.command_sender
			.send(MixerCommand::RemoveEffect(self.index, id).into())
			.map_err(|_| TrackHandleError::BackendDisconnected)
	}
}
