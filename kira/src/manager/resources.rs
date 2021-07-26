use atomic_arena::{Arena, Controller};
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::sound::Sound;

use self::sounds::Sounds;

use super::AudioManagerSettings;

pub mod sounds;

pub(super) struct UnusedResourceProducers {
	pub sound: Producer<Sound>,
}

pub(super) struct UnusedResourceConsumers {
	pub sound: Consumer<Sound>,
}

pub(super) fn create_unused_resource_channels(
	settings: &AudioManagerSettings,
) -> (UnusedResourceProducers, UnusedResourceConsumers) {
	let (unused_sound_producer, unused_sound_consumer) =
		RingBuffer::new(settings.sound_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
		},
		UnusedResourceConsumers {
			sound: unused_sound_consumer,
		},
	)
}

pub(super) struct Resources {
	pub sounds: Sounds,
}

pub(super) struct ResourceControllers {
	pub sound_controller: Controller,
}

pub(super) fn create_resources(
	settings: &AudioManagerSettings,
	unused_resource_producers: UnusedResourceProducers,
) -> (Resources, ResourceControllers) {
	let sounds = Sounds {
		sounds: Arena::new(settings.sound_capacity),
		unused_sound_producer: unused_resource_producers.sound,
	};
	let sound_controller = sounds.sounds.controller();
	(
		Resources { sounds },
		ResourceControllers { sound_controller },
	)
}
