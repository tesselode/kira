mod backend;
pub mod event;
mod sound;
pub mod sound_bank;
mod stereo_sample;

use backend::Backend;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use event::Command;
use ringbuf::Producer;
use sound_bank::{SoundBank, SoundId};
use std::error::Error;

const COMMAND_BUFFER_CAPACITY: usize = 100;

pub enum SoundName {
	Test,
}

pub struct AudioManager {
	command_producer: Producer<Command>,
	stream: Stream,
}

impl AudioManager {
	pub fn new(sound_bank: SoundBank) -> Result<Self, Box<dyn Error>> {
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
		let (command_producer, command_consumer) =
			ringbuf::RingBuffer::new(COMMAND_BUFFER_CAPACITY).split();
		let mut backend = Backend::new(sample_rate, sound_bank, command_consumer);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					frame[0] = out.left;
					frame[1] = out.right;
				}
			},
			move |err| {},
		)?;
		stream.play()?;
		Ok(Self {
			stream,
			command_producer,
		})
	}

	pub fn play_sound(&mut self, sound_id: SoundId) {
		match self.command_producer.push(Command::PlaySound(sound_id)) {
			Ok(_) => {}
			Err(_) => {}
		}
	}
}
