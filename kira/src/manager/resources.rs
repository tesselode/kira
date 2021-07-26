pub mod instances;
pub mod sounds;

use atomic_arena::{Arena, Controller};
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::sound::{instance::Instance, Sound};

use self::{instances::Instances, sounds::Sounds};

use super::AudioManagerSettings;

pub(super) struct UnusedResourceProducers {
	pub sound: Producer<Sound>,
	pub instance: Producer<Instance>,
}

pub(super) struct UnusedResourceConsumers {
	pub sound: Consumer<Sound>,
	pub instance: Consumer<Instance>,
}

pub(super) fn create_unused_resource_channels(
	settings: &AudioManagerSettings,
) -> (UnusedResourceProducers, UnusedResourceConsumers) {
	let (unused_sound_producer, unused_sound_consumer) =
		RingBuffer::new(settings.sound_capacity).split();
	let (unused_instance_producer, unused_instance_consumer) =
		RingBuffer::new(settings.instance_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
			instance: unused_instance_producer,
		},
		UnusedResourceConsumers {
			sound: unused_sound_consumer,
			instance: unused_instance_consumer,
		},
	)
}

pub(super) struct Resources {
	pub sounds: Sounds,
	pub instances: Instances,
}

pub(super) struct ResourceControllers {
	pub sound_controller: Controller,
	pub instance_controller: Controller,
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
	let instances = Instances {
		instances: Arena::new(settings.instance_capacity),
		unused_instance_producer: unused_resource_producers.instance,
	};
	let instance_controller = instances.instances.controller();
	(
		Resources { sounds, instances },
		ResourceControllers {
			sound_controller,
			instance_controller,
		},
	)
}
