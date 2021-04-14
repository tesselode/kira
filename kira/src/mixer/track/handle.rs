use basedrop::{Handle, Owned, Shared};
use ringbuf::Producer;

use crate::{
	error::CommandQueueFullError,
	mixer::{
		effect::{Effect, EffectSettings},
		effect_slot::{EffectSlot, EffectSlotState},
	},
};

use super::TrackInput;

pub struct TrackHandle {
	input: TrackInput,
	effect_slot_producer: Producer<EffectSlot>,
	collector_handle: Handle,
	sample_rate: u32,
}

impl TrackHandle {
	pub(crate) fn new(
		input: TrackInput,
		effect_slot_producer: Producer<EffectSlot>,
		collector_handle: Handle,
		sample_rate: u32,
	) -> Self {
		Self {
			input,
			effect_slot_producer,
			collector_handle,
			sample_rate,
		}
	}

	pub(crate) fn input(&self) -> TrackInput {
		self.input.clone()
	}

	pub fn add_effect(
		&mut self,
		effect: impl Effect + 'static,
		settings: EffectSettings,
	) -> Result<(), CommandQueueFullError> {
		let mut effect: Owned<Box<dyn Effect>> = Owned::new(&self.collector_handle, Box::new(effect));
		effect.init(self.sample_rate);
		let effect_slot = EffectSlot::new(
			effect,
			settings.mix,
			Shared::new(
				&self.collector_handle,
				EffectSlotState::new(settings.enabled),
			),
		);
		self.effect_slot_producer
			.push(effect_slot)
			.map_err(|_| CommandQueueFullError)?;
		Ok(())
	}
}
