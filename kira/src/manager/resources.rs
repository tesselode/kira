pub mod instances;
pub mod parameters;
pub mod sounds;

use atomic_arena::Controller;
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{
	parameter::Parameter,
	sound::{instance::Instance, Sound},
};

use self::{instances::Instances, parameters::Parameters, sounds::Sounds};

use super::AudioManagerSettings;

pub(super) struct UnusedResourceProducers {
	pub sound: Producer<Sound>,
	pub instance: Producer<Instance>,
	pub parameter: Producer<Parameter>,
}

pub(super) struct UnusedResourceConsumers {
	pub sound: Consumer<Sound>,
	pub instance: Consumer<Instance>,
	pub parameter: Consumer<Parameter>,
}

pub(super) fn create_unused_resource_channels(
	settings: &AudioManagerSettings,
) -> (UnusedResourceProducers, UnusedResourceConsumers) {
	let (unused_sound_producer, unused_sound_consumer) =
		RingBuffer::new(settings.sound_capacity).split();
	let (unused_instance_producer, unused_instance_consumer) =
		RingBuffer::new(settings.instance_capacity).split();
	let (unused_parameter_producer, unused_parameter_consumer) =
		RingBuffer::new(settings.parameter_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
			instance: unused_instance_producer,
			parameter: unused_parameter_producer,
		},
		UnusedResourceConsumers {
			sound: unused_sound_consumer,
			instance: unused_instance_consumer,
			parameter: unused_parameter_consumer,
		},
	)
}

pub(super) struct Resources {
	pub sounds: Sounds,
	pub instances: Instances,
	pub parameters: Parameters,
}

pub(super) struct ResourceControllers {
	pub sound_controller: Controller,
	pub instance_controller: Controller,
	pub parameter_controller: Controller,
}

pub(super) fn create_resources(
	settings: &AudioManagerSettings,
	unused_resource_producers: UnusedResourceProducers,
) -> (Resources, ResourceControllers) {
	let sounds = Sounds::new(settings.sound_capacity, unused_resource_producers.sound);
	let sound_controller = sounds.controller();
	let instances = Instances::new(
		settings.instance_capacity,
		unused_resource_producers.instance,
	);
	let instance_controller = instances.controller();
	let parameters = Parameters::new(
		settings.parameter_capacity,
		unused_resource_producers.parameter,
	);
	let parameter_controller = parameters.controller();
	(
		Resources {
			sounds,
			instances,
			parameters,
		},
		ResourceControllers {
			sound_controller,
			instance_controller,
			parameter_controller,
		},
	)
}
