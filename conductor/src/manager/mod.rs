mod backend;

use crate::project::{Project, SoundId};
use backend::{Backend, Command};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::error::Error;

const COMMAND_QUEUE_CAPACITY: usize = 100;
const EVENT_QUEUE_CAPACITY: usize = 100;

#[derive(Debug)]
pub enum Event {
	MetronomeInterval(f32),
}

pub struct AudioManagerSettings {
	pub num_instances: usize,
	pub tempo: f32,
	pub metronome_event_intervals: Vec<f32>,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_instances: 100,
			tempo: 120.0,
			metronome_event_intervals: vec![1.0],
		}
	}
}

pub struct AudioManager {
	command_producer: Producer<Command>,
	event_consumer: Consumer<Event>,
	_stream: Stream,
}

impl AudioManager {
	pub fn new(project: Project, settings: AudioManagerSettings) -> Result<Self, Box<dyn Error>> {
		let host = cpal::default_host();
		let device = host.default_output_device().unwrap();
		let mut supported_configs_range = device.supported_output_configs().unwrap();
		let supported_config = supported_configs_range
			.next()
			.unwrap()
			.with_max_sample_rate();
		let config = supported_config.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let (command_producer, command_consumer) = RingBuffer::new(COMMAND_QUEUE_CAPACITY).split();
		let (event_producer, event_consumer) = RingBuffer::new(EVENT_QUEUE_CAPACITY).split();
		let mut backend = Backend::new(
			sample_rate,
			project,
			settings,
			command_consumer,
			event_producer,
		);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					frame[0] = out.left;
					frame[1] = out.right;
				}
			},
			move |_| {},
		)?;
		stream.play()?;
		Ok(Self {
			command_producer,
			event_consumer,
			_stream: stream,
		})
	}

	pub fn play_sound(&mut self, sound_id: SoundId) {
		match self.command_producer.push(Command::PlaySound(sound_id)) {
			Ok(_) => {}
			Err(_) => {}
		}
	}

	pub fn start_metronome(&mut self) {
		match self.command_producer.push(Command::StartMetronome) {
			Ok(_) => {}
			Err(_) => {}
		}
	}

	pub fn events(&mut self) -> Vec<Event> {
		let mut events = vec![];
		while let Some(event) = self.event_consumer.pop() {
			events.push(event);
		}
		events
	}
}
