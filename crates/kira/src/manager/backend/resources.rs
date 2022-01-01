pub(crate) mod mixer;
pub(crate) mod sounds;

use std::sync::Arc;

use atomic_arena::Controller;
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{
	clock::{clocks::Clocks, Clock},
	manager::AudioManagerSettings,
	sound::Sound,
	track::Track,
};

use self::{mixer::Mixer, sounds::Sounds};

use super::context::Context;

pub(crate) struct UnusedResourceProducers {
	pub sound: Producer<Box<dyn Sound>>,
	pub sub_track: Producer<Track>,
	pub clock: Producer<Clock>,
}

/// Collects resources that have been discarded by
/// a [`Renderer`](super::Renderer) to be
/// deallocated at an appropriate time.
pub struct UnusedResourceCollector {
	unused_sound_consumer: Consumer<Box<dyn Sound>>,
	unused_sub_track_consumer: Consumer<Track>,
	unused_clock_consumer: Consumer<Clock>,
}

impl UnusedResourceCollector {
	/// Deallocates all unused resources that have been collected.
	pub fn drain(&mut self) {
		while self.unused_sound_consumer.pop().is_some() {}
		while self.unused_sub_track_consumer.pop().is_some() {}
		while self.unused_clock_consumer.pop().is_some() {}
	}
}

pub(crate) fn create_unused_resource_channels(
	settings: &AudioManagerSettings,
) -> (UnusedResourceProducers, UnusedResourceCollector) {
	let (unused_sound_producer, unused_sound_consumer) =
		RingBuffer::new(settings.sound_capacity).split();
	let (unused_sub_track_producer, unused_sub_track_consumer) =
		RingBuffer::new(settings.sub_track_capacity).split();
	let (unused_clock_producer, unused_clock_consumer) =
		RingBuffer::new(settings.clock_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
			sub_track: unused_sub_track_producer,
			clock: unused_clock_producer,
		},
		UnusedResourceCollector {
			unused_sound_consumer,
			unused_sub_track_consumer,
			unused_clock_consumer,
		},
	)
}

pub(crate) struct Resources {
	pub sounds: Sounds,
	pub mixer: Mixer,
	pub clocks: Clocks,
}

pub(crate) struct ResourceControllers {
	pub sound_controller: Controller,
	pub sub_track_controller: Controller,
	pub clock_controller: Controller,
}

pub(crate) fn create_resources(
	settings: AudioManagerSettings,
	unused_resource_producers: UnusedResourceProducers,
	context: &Arc<Context>,
) -> (Resources, ResourceControllers) {
	let sounds = Sounds::new(settings.sound_capacity, unused_resource_producers.sound);
	let sound_controller = sounds.controller();
	let mixer = Mixer::new(
		settings.sub_track_capacity,
		unused_resource_producers.sub_track,
		context,
		settings.main_track_builder,
	);
	let sub_track_controller = mixer.sub_track_controller();
	let clocks = Clocks::new(settings.clock_capacity, unused_resource_producers.clock);
	let clock_controller = clocks.controller();
	(
		Resources {
			sounds,
			mixer,
			clocks,
		},
		ResourceControllers {
			sound_controller,
			sub_track_controller,
			clock_controller,
		},
	)
}
