//! An interface for controlling mixer tracks.

use basedrop::Owned;
use indexmap::IndexSet;
use thiserror::Error;

use crate::{
	command::{
		producer::{CommandError, CommandProducer},
		MixerCommand,
	},
	mixer::effect::{handle::EffectHandle, Effect, EffectId, EffectSettings},
};

use super::{TrackIndex, TrackSettings};

/// Something that can go wrong when using a [`TrackHandle`] to
/// add an effect to a mixer track.
#[derive(Debug, Error)]
pub enum AddEffectError {
	/// The maximum effect limit for this track has been reached.
	#[error(
		"Cannot add an effect because the max number of effects for this track has been reached"
	)]
	EffectLimitReached,
	/// No effect with the specified ID exists on this track.
	#[error("No effect with the specified ID exists on this track")]
	NoEffectWithId(EffectId),
	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Something that can go wrong when using a [`TrackHandle`] to
/// remove an effect from a mixer track.
#[derive(Debug, Error)]
pub enum RemoveEffectError {
	/// No effect with the specified ID exists on this track.
	#[error("No effect with the specified ID exists on this track")]
	NoEffectWithId(EffectId),
	/// A command could not be sent to the audio thread.
	#[error("Could not send the command to the audio thread.")]
	CommandProducerError(#[from] CommandError),
}

/// Allows you to control a mixer sound.
pub struct TrackHandle {
	index: TrackIndex,
	command_producer: CommandProducer,
	active_effect_ids: IndexSet<EffectId>,
	resource_collector_handle: basedrop::Handle,
}

impl TrackHandle {
	pub(crate) fn new(
		index: TrackIndex,
		settings: &TrackSettings,
		command_producer: CommandProducer,
		resource_collector_handle: basedrop::Handle,
	) -> Self {
		Self {
			index,
			command_producer,
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
	) -> Result<EffectHandle, AddEffectError> {
		if self.active_effect_ids.len() >= self.active_effect_ids.capacity() {
			return Err(AddEffectError::EffectLimitReached);
		}
		let id = settings.id;
		let handle = EffectHandle::new(self.index, &settings, self.command_producer.clone());
		self.command_producer.push(
			MixerCommand::AddEffect(
				self.index,
				Owned::new(&self.resource_collector_handle, Box::new(effect)),
				settings,
			)
			.into(),
		)?;
		self.active_effect_ids.insert(id);
		Ok(handle)
	}

	/// Removes an effect from the track.
	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), RemoveEffectError> {
		let id = id.into();
		if !self.active_effect_ids.remove(&id) {
			return Err(RemoveEffectError::NoEffectWithId(id));
		}
		self.command_producer
			.push(MixerCommand::RemoveEffect(self.index, id).into())?;
		Ok(())
	}
}
