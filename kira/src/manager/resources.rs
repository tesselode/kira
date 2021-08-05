pub(crate) mod audio_streams;
pub(crate) mod clocks;
pub(crate) mod instances;
pub(crate) mod mixer;
mod parameters;
pub(crate) mod sounds;

pub use parameters::*;

use std::sync::Arc;

use atomic_arena::Controller;
use ringbuf::{Consumer, Producer, RingBuffer};

use crate::{
	audio_stream::AudioStreamWrapper,
	clock::Clock,
	parameter::Parameter,
	sound::{instance::Instance, wrapper::SoundWrapper},
	track::Track,
};

use self::{
	audio_streams::AudioStreams, clocks::Clocks, instances::Instances, mixer::Mixer, sounds::Sounds,
};

use super::{renderer::context::Context, AudioManagerSettings};

pub(super) struct UnusedResourceProducers {
	pub sound: Producer<SoundWrapper>,
	pub instance: Producer<Instance>,
	pub parameter: Producer<Parameter>,
	pub sub_track: Producer<Track>,
	pub clock: Producer<Clock>,
	pub audio_stream: Producer<AudioStreamWrapper>,
}

pub struct UnusedResourceCollector {
	unused_sound_consumer: Consumer<SoundWrapper>,
	unused_instance_consumer: Consumer<Instance>,
	unused_parameter_consumer: Consumer<Parameter>,
	unused_sub_track_consumer: Consumer<Track>,
	unused_clock_consumer: Consumer<Clock>,
	unused_audio_stream_consumer: Consumer<AudioStreamWrapper>,
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
		while self.unused_clock_consumer.pop().is_some() {
			println!("dropped clock");
		}
		while self.unused_audio_stream_consumer.pop().is_some() {
			println!("dropped audio stream");
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
	let (unused_clock_producer, unused_clock_consumer) =
		RingBuffer::new(settings.clock_capacity).split();
	let (unused_audio_stream_producer, unused_audio_stream_consumer) =
		RingBuffer::new(settings.audio_stream_capacity).split();
	(
		UnusedResourceProducers {
			sound: unused_sound_producer,
			instance: unused_instance_producer,
			parameter: unused_parameter_producer,
			sub_track: unused_sub_track_producer,
			clock: unused_clock_producer,
			audio_stream: unused_audio_stream_producer,
		},
		UnusedResourceCollector {
			unused_sound_consumer,
			unused_instance_consumer,
			unused_parameter_consumer,
			unused_sub_track_consumer,
			unused_clock_consumer,
			unused_audio_stream_consumer,
		},
	)
}

pub(super) struct Resources {
	pub sounds: Sounds,
	pub instances: Instances,
	pub parameters: Parameters,
	pub mixer: Mixer,
	pub clocks: Clocks,
	pub audio_streams: AudioStreams,
}

pub(super) struct ResourceControllers {
	pub sound_controller: Controller,
	pub instance_controller: Controller,
	pub parameter_controller: Controller,
	pub sub_track_controller: Controller,
	pub clock_controller: Controller,
	pub audio_stream_controller: Controller,
}

pub(super) fn create_resources(
	settings: &AudioManagerSettings,
	unused_resource_producers: UnusedResourceProducers,
	context: &Arc<Context>,
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
		context,
	);
	let sub_track_controller = mixer.sub_track_controller();
	let clocks = Clocks::new(settings.clock_capacity, unused_resource_producers.clock);
	let clock_controller = clocks.controller();
	let audio_streams = AudioStreams::new(
		settings.audio_stream_capacity,
		unused_resource_producers.audio_stream,
	);
	let audio_stream_controller = audio_streams.controller();
	(
		Resources {
			sounds,
			instances,
			parameters,
			mixer,
			clocks,
			audio_streams,
		},
		ResourceControllers {
			sound_controller,
			instance_controller,
			parameter_controller,
			sub_track_controller,
			clock_controller,
			audio_stream_controller,
		},
	)
}
