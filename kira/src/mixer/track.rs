pub mod handle;
pub mod routes;
pub mod settings;

use atomig::{Atomic, Ordering};
use basedrop::{Handle, Owned, Shared};
use ringbuf::Consumer;

use crate::{value::Value, Frame};

use super::effect_slot::EffectSlot;

#[derive(Clone)]
pub(crate) struct TrackInput(Shared<Atomic<Frame>>);

impl TrackInput {
	fn new(collector_handle: &Handle) -> Self {
		Self(Shared::new(
			collector_handle,
			Atomic::new(Frame::from_mono(0.0)),
		))
	}

	pub(crate) fn add(&self, frame: Frame) {
		let previous = self.0.load(Ordering::SeqCst);
		self.0.store(previous + frame, Ordering::SeqCst);
	}

	fn take(&self) -> Frame {
		self.0.swap(Frame::from_mono(0.0), Ordering::SeqCst)
	}
}

pub(crate) struct Track {
	input: TrackInput,
	routes: Vec<(TrackInput, f64)>,
	volume: Value<f64>,
	effect_slots: Vec<EffectSlot>,
	effect_slot_consumer: Consumer<EffectSlot>,
}

impl Track {
	pub fn new(
		collector_handle: &Handle,
		routes: Vec<(TrackInput, f64)>,
		volume: Value<f64>,
		effect_capacity: usize,
		effect_slot_consumer: Consumer<EffectSlot>,
	) -> Self {
		Self {
			input: TrackInput::new(collector_handle),
			routes,
			volume,
			effect_slots: Vec::with_capacity(effect_capacity),
			effect_slot_consumer,
		}
	}

	pub fn input(&self) -> &TrackInput {
		&self.input
	}

	pub fn process(&mut self, dt: f64) -> Frame {
		while let Some(effect_slot) = self.effect_slot_consumer.pop() {
			if self.effect_slots.len() < self.effect_slots.capacity() {
				self.effect_slots.push(effect_slot);
			}
		}

		let mut out = self.input.take() * self.volume.get() as f32;
		for effect_slot in &mut self.effect_slots {
			out = effect_slot.process(out, dt);
		}
		for (input, level) in &self.routes {
			input.add(out * *level as f32);
		}
		out
	}
}
