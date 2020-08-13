use crate::{
	command::{Command, SoundCommand},
	error::ConductorError,
	sound::{Sound, SoundId},
};
use backend::Backend;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Producer, RingBuffer};
use std::{error::Error, path::Path};

mod backend;

/// Settings for an `AudioManager`.
pub struct AudioManagerSettings {
	/// The number of commands that be sent to the audio thread at a time.
	///
	/// Each action you take, like starting an instance or pausing a sequence,
	/// queues up one command.
	pub num_commands: usize,
	/// The maximum number of sounds that can be loaded at once.
	pub num_sounds: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_sounds: 100,
		}
	}
}

/**
Plays and manages audio.

The `AudioManager` is responsible for all communication between the gameplay thread
and the audio thread.
*/
pub struct AudioManager {
	command_producer: Producer<Command>,
	//event_consumer: Consumer<Event>,
	_stream: Stream,
}

impl AudioManager {
	/// Creates a new audio manager and starts an audio thread.
	///
	/// The `Project` is given to the audio thread, so make sure you've loaded all
	/// the sounds you want to use before you create an `AudioManager`.
	pub fn new(settings: AudioManagerSettings) -> Result<Self, Box<dyn Error>> {
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
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		//let (event_producer, event_consumer) = RingBuffer::new(settings.num_events).split();
		let mut backend = Backend::new(
			//sample_rate,
			//project,
			settings,
			command_consumer,
			//event_producer,
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
			//event_consumer,
			_stream: stream,
		})
	}

	pub fn load_sound(&mut self, path: &Path) -> Result<SoundId, Box<dyn Error>> {
		let sound = Sound::from_ogg_file(path)?;
		let id = SoundId::new(sound.duration());
		match self
			.command_producer
			.push(Command::Sound(SoundCommand::LoadSound(id, sound)))
		{
			Ok(_) => Ok(id),
			Err(_) => Err(Box::new(ConductorError::SendCommand)),
		}
	}
}
