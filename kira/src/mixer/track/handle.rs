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

use super::{
	SendTrackId, SendTrackSettings, SubTrackId, SubTrackSettings, TrackIndex,
	MAIN_TRACK_NUM_EFFECTS,
};

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

/// Allows you to control the main mixer track.
pub struct MainTrackHandle {
	command_producer: CommandProducer,
	active_effect_ids: IndexSet<EffectId>,
	sample_rate: u32,
	resource_collector_handle: basedrop::Handle,
}

impl MainTrackHandle {
	pub(crate) fn new(
		command_producer: CommandProducer,
		sample_rate: u32,
		resource_collector_handle: basedrop::Handle,
	) -> Self {
		Self {
			command_producer,
			active_effect_ids: IndexSet::with_capacity(MAIN_TRACK_NUM_EFFECTS),
			sample_rate,
			resource_collector_handle,
		}
	}

	/// Adds an effect to the track.
	pub fn add_effect(
		&mut self,
		mut effect: impl Effect + 'static,
		settings: EffectSettings,
	) -> Result<EffectHandle, AddEffectError> {
		if self.active_effect_ids.len() >= self.active_effect_ids.capacity() {
			return Err(AddEffectError::EffectLimitReached);
		}
		let effect_id = settings.id.unwrap_or(EffectId::new());
		let handle = EffectHandle::new(
			effect_id,
			TrackIndex::Main,
			&settings,
			self.command_producer.clone(),
		);
		effect.init(self.sample_rate);
		self.command_producer.push(
			MixerCommand::AddEffect(
				TrackIndex::Main,
				effect_id,
				Owned::new(&self.resource_collector_handle, Box::new(effect)),
				settings,
			)
			.into(),
		)?;
		self.active_effect_ids.insert(effect_id);
		Ok(handle)
	}

	/// Removes an effect from the track.
	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), RemoveEffectError> {
		let id = id.into();
		if !self.active_effect_ids.remove(&id) {
			return Err(RemoveEffectError::NoEffectWithId(id));
		}
		self.command_producer
			.push(MixerCommand::RemoveEffect(TrackIndex::Main, id).into())?;
		Ok(())
	}
}

/// Allows you to control a mixer sub-track.
pub struct SubTrackHandle {
	id: SubTrackId,
	command_producer: CommandProducer,
	active_effect_ids: IndexSet<EffectId>,
	sample_rate: u32,
	resource_collector_handle: basedrop::Handle,
}

impl SubTrackHandle {
	pub(crate) fn new(
		id: SubTrackId,
		settings: &SubTrackSettings,
		command_producer: CommandProducer,
		sample_rate: u32,
		resource_collector_handle: basedrop::Handle,
	) -> Self {
		Self {
			id,
			command_producer,
			active_effect_ids: IndexSet::with_capacity(settings.num_effects),
			sample_rate,
			resource_collector_handle,
		}
	}

	/// Gets the track that this handle controls.
	pub fn id(&self) -> SubTrackId {
		self.id
	}

	/// Adds an effect to the track.
	pub fn add_effect(
		&mut self,
		mut effect: impl Effect + 'static,
		settings: EffectSettings,
	) -> Result<EffectHandle, AddEffectError> {
		if self.active_effect_ids.len() >= self.active_effect_ids.capacity() {
			return Err(AddEffectError::EffectLimitReached);
		}
		let effect_id = settings.id.unwrap_or(EffectId::new());
		let handle = EffectHandle::new(
			effect_id,
			self.id.into(),
			&settings,
			self.command_producer.clone(),
		);
		effect.init(self.sample_rate);
		self.command_producer.push(
			MixerCommand::AddEffect(
				self.id.into(),
				effect_id,
				Owned::new(&self.resource_collector_handle, Box::new(effect)),
				settings,
			)
			.into(),
		)?;
		self.active_effect_ids.insert(effect_id);
		Ok(handle)
	}

	/// Removes an effect from the track.
	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), RemoveEffectError> {
		let id = id.into();
		if !self.active_effect_ids.remove(&id) {
			return Err(RemoveEffectError::NoEffectWithId(id));
		}
		self.command_producer
			.push(MixerCommand::RemoveEffect(self.id.into(), id).into())?;
		Ok(())
	}
}

/// Allows you to control a mixer send track.
pub struct SendTrackHandle {
	id: SendTrackId,
	command_producer: CommandProducer,
	active_effect_ids: IndexSet<EffectId>,
	sample_rate: u32,
	resource_collector_handle: basedrop::Handle,
}

impl SendTrackHandle {
	pub(crate) fn new(
		id: SendTrackId,
		settings: &SendTrackSettings,
		command_producer: CommandProducer,
		sample_rate: u32,
		resource_collector_handle: basedrop::Handle,
	) -> Self {
		Self {
			id,
			command_producer,
			active_effect_ids: IndexSet::with_capacity(settings.num_effects),
			sample_rate,
			resource_collector_handle,
		}
	}

	/// Gets the track that this handle controls.
	pub fn id(&self) -> SendTrackId {
		self.id
	}

	/// Adds an effect to the track.
	pub fn add_effect(
		&mut self,
		mut effect: impl Effect + 'static,
		settings: EffectSettings,
	) -> Result<EffectHandle, AddEffectError> {
		if self.active_effect_ids.len() >= self.active_effect_ids.capacity() {
			return Err(AddEffectError::EffectLimitReached);
		}
		let effect_id = settings.id.unwrap_or(EffectId::new());
		let handle = EffectHandle::new(
			effect_id,
			self.id.into(),
			&settings,
			self.command_producer.clone(),
		);
		effect.init(self.sample_rate);
		self.command_producer.push(
			MixerCommand::AddEffect(
				self.id.into(),
				effect_id,
				Owned::new(&self.resource_collector_handle, Box::new(effect)),
				settings,
			)
			.into(),
		)?;
		self.active_effect_ids.insert(effect_id);
		Ok(handle)
	}

	/// Removes an effect from the track.
	pub fn remove_effect(&mut self, id: impl Into<EffectId>) -> Result<(), RemoveEffectError> {
		let id = id.into();
		if !self.active_effect_ids.remove(&id) {
			return Err(RemoveEffectError::NoEffectWithId(id));
		}
		self.command_producer
			.push(MixerCommand::RemoveEffect(self.id.into(), id).into())?;
		Ok(())
	}
}
