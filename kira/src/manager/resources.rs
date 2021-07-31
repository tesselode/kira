pub mod instances;
pub mod mixer;
pub mod parameters;
pub mod sounds;

use atomic_arena::Controller;
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{
	parameter::Parameter,
	sound::{instance::Instance, Sound},
	track::Track,
};

use self::{instances::Instances, mixer::Mixer, parameters::Parameters, sounds::Sounds};

use super::AudioManagerSettings;

pub(super) struct UnusedResourceProducers {
	pub sound: Producer<Sound>,
	pub instance: Producer<Instance>,
	pub parameter: Producer<Parameter>,
	pub sub_track: Producer<Track>,
}

pub struct UnusedResourceCollector {
	unused_sound_consumer: Consumer<Sound>,
	unused_instance_consumer: Consumer<Instance>,
	unused_parameter_consumer: Consumer<Parameter>,
	unused_sub_track_consumer: Consumer<Track>,
}

impl UnusedResourceCollector {
	pub fn drain(&mut self) {
		while self.unused_sound_consumer.pop().is_some() {
			println!("dropped sound");
		}
		while self.unused_instance_consumer.pop().is_some() {
			println!("dropped instance");
		}
		while self.unused_parameter_consumer.pop().is_some() {
			println!("dropped parameter");
		}
		while self.unused_sub_track_consumer.pop().is_some() {
			println!("dropped sub-track");
		}
	}
}

pub(super) fn create_unused_resource_channels(
	settings: &AudioManagerSettings,
) -> (UnusedResourceProducers, UnusedResourceCollector) {
	let (unused_sound_producer, unused_sound_consumer) =
		RingBuffer::new(settings.sound_capacity).split();
	let (unused_instance_producer, unused_instance_consumer) =
		RingBuffer::new(settings.instance_capacity).split();
	let (unused_parameter_producer, unused_parameter_consumer) =
		RingBuffer::new(settings.parameter_capacity).split();
	let (unused_sub_track_producer, unused_sub_track_consumer) =
		RingBuffer::new(settings.sub_track_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
			instance: unused_instance_producer,
			parameter: unused_parameter_producer,
			sub_track: unused_sub_track_producer,
		},
		UnusedResourceCollector {
			unused_sound_consumer,
			unused_instance_consumer,
			unused_parameter_consumer,
			unused_sub_track_consumer,
		},
	)
}

pub(super) struct Resources {
	pub sounds: Sounds,
	pub instances: Instances,
	pub parameters: Parameters,
	pub mixer: Mixer,
}

pub(super) struct ResourceControllers {
	pub sound_controller: Controller,
	pub instance_controller: Controller,
	pub parameter_controller: Controller,
	pub sub_track_controller: Controller,
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
	let mixer = Mixer::new(
		settings.sub_track_capacity,
		unused_resource_producers.sub_track,
	);
	let sub_track_controller = mixer.sub_track_controller();
	(
		Resources {
			sounds,
			instances,
			parameters,
			mixer,
		},
		ResourceControllers {
			sound_controller,
			instance_controller,
			parameter_controller,
			sub_track_controller,
		},
	)
}
