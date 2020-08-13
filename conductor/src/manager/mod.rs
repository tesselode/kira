use crate::{
	command::{Command, InstanceCommand, SoundCommand},
	error::ConductorError,
	instance::{InstanceId, InstanceSettings},
	sound::{Sound, SoundId, SoundMetadata},
	tween::Tween,
};
use backend::Backend;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Consumer, Producer, RingBuffer};
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
	/// The maximum number of instances of sounds that can be playing at once.
	pub num_instances: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_sounds: 100,
			num_instances: 100,
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
	sounds_to_unload_consumer: Consumer<Sound>,
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
		let (sounds_to_unload_producer, sounds_to_unload_consumer) =
			RingBuffer::new(settings.num_sounds).split();
		//let (event_producer, event_consumer) = RingBuffer::new(settings.num_events).split();
		let mut backend = Backend::new(
			sample_rate,
			//project,
			settings,
			command_consumer,
			sounds_to_unload_producer,
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
			sounds_to_unload_consumer,
			_stream: stream,
		})
	}

	/// Loads a sound from a file path.
	///
	/// Returns a handle to the sound. Keep this so you can play the sound later.
	pub fn load_sound(
		&mut self,
		path: &Path,
		metadata: SoundMetadata,
	) -> Result<SoundId, Box<dyn Error>> {
		let sound = Sound::from_ogg_file(path)?;
		let id = SoundId::new(sound.duration(), metadata);
		match self
			.command_producer
			.push(Command::Sound(SoundCommand::LoadSound(id, sound)))
		{
			Ok(_) => Ok(id),
			Err(_) => Err(Box::new(ConductorError::SendCommand)),
		}
	}

	/// Unloads a sound, deallocating its memory.
	pub fn unload_sound(&mut self, id: SoundId) -> Result<(), Box<dyn Error>> {
		match self
			.command_producer
			.push(Command::Sound(SoundCommand::UnloadSound(id)))
		{
			Ok(_) => Ok(()),
			Err(_) => Err(Box::new(ConductorError::SendCommand)),
		}
	}

	/// Plays a sound.
	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: InstanceSettings,
	) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::PlaySound(
				sound_id,
				instance_id,
				settings,
			))) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	pub fn set_instance_volume(
		&mut self,
		id: InstanceId,
		volume: f32,
		tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::SetInstanceVolume(
				id, volume, tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	pub fn set_instance_pitch(
		&mut self,
		id: InstanceId,
		pitch: f32,
		tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::SetInstancePitch(
				id, pitch, tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Pauses a currently playing instance of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn pause_instance(
		&mut self,
		instance_id: InstanceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::PauseInstance(
				instance_id,
				fade_tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Resumes a currently paused instance of a sound.
	///
	/// You can optionally provide a fade-in duration (in seconds).
	pub fn resume_instance(
		&mut self,
		instance_id: InstanceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::ResumeInstance(
				instance_id,
				fade_tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Stops a currently playing instance of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds). Once the
	/// instance is stopped, it cannot be restarted.
	pub fn stop_instance(
		&mut self,
		instance_id: InstanceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::StopInstance(
				instance_id,
				fade_tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Pauses all currently playing instances of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn pause_instances_of_sound(
		&mut self,
		sound_id: SoundId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::PauseInstancesOfSound(
				sound_id, fade_tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Resumes all currently playing instances of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn resume_instances_of_sound(
		&mut self,
		sound_id: SoundId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self.command_producer.push(Command::Instance(
			InstanceCommand::ResumeInstancesOfSound(sound_id, fade_tween),
		)) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Stops all currently playing instances of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn stop_instances_of_sound(
		&mut self,
		sound_id: SoundId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Instance(InstanceCommand::StopInstancesOfSound(
				sound_id, fade_tween,
			))) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	pub fn events(&mut self) {
		// unload sounds on the main thread
		while let Some(_) = self.sounds_to_unload_consumer.pop() {}
	}
}
