use crate::{
	command::{Command, InstanceCommand, MetronomeCommand, SequenceCommand, SoundCommand},
	error::ConductorError,
	instance::{InstanceId, InstanceSettings},
	metronome::MetronomeSettings,
	sequence::{Sequence, SequenceId},
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

/// Events that can be sent by the audio thread.
#[derive(Debug)]
pub enum Event {
	/**
	Sent when the metronome passes a certain interval (in beats).

	For example, an event with an interval of `1.0` will be sent
	every beat, and an event with an interval of `0.25` will be
	sent every sixteenth note (one quarter of a beat).

	The intervals that a metronome emits events for are defined
	when the metronome is created.
	*/
	MetronomeIntervalPassed(f32),
}

/// Settings for an `AudioManager`.
pub struct AudioManagerSettings {
	/// The number of commands that be sent to the audio thread at a time.
	///
	/// Each action you take, like starting an instance or pausing a sequence,
	/// queues up one command.
	pub num_commands: usize,
	/// The number of events the audio thread can send at a time.
	pub num_events: usize,
	/// The maximum number of sounds that can be loaded at once.
	pub num_sounds: usize,
	/// The maximum number of instances of sounds that can be playing at once.
	pub num_instances: usize,
	/// The maximum number of sequences that can be running at a time.
	pub num_sequences: usize,
	/// Settings for the metronome.
	pub metronome_settings: MetronomeSettings,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_events: 100,
			num_sounds: 100,
			num_instances: 100,
			num_sequences: 25,
			metronome_settings: MetronomeSettings::default(),
		}
	}
}

/**
Plays and manages audio.

The `AudioManager` is responsible for all communication between the gameplay thread
and the audio thread.
*/
pub struct AudioManager {
	quit_signal_producer: Producer<bool>,
	command_producer: Producer<Command>,
	event_consumer: Consumer<Event>,
	sounds_to_unload_consumer: Consumer<Sound>,
	sequences_to_unload_consumer: Consumer<Sequence>,
}

impl AudioManager {
	/// Creates a new audio manager and starts an audio thread.
	pub fn new(settings: AudioManagerSettings) -> Result<Self, Box<dyn Error>> {
		let (quit_signal_producer, mut quit_signal_consumer) = RingBuffer::new(1).split();
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let (sounds_to_unload_producer, sounds_to_unload_consumer) =
			RingBuffer::new(settings.num_sounds).split();
		let (sequences_to_unload_producer, sequences_to_unload_consumer) =
			RingBuffer::new(settings.num_sequences).split();
		let (event_producer, event_consumer) = RingBuffer::new(settings.num_events).split();
		std::thread::spawn(move || {
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
			let mut backend = Backend::new(
				sample_rate,
				settings,
				command_consumer,
				event_producer,
				sounds_to_unload_producer,
				sequences_to_unload_producer,
			);
			let stream = device
				.build_output_stream(
					&config,
					move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
						for frame in data.chunks_exact_mut(channels as usize) {
							let out = backend.process();
							frame[0] = out.left;
							frame[1] = out.right;
						}
					},
					move |_| {},
				)
				.unwrap();
			stream.play().unwrap();
			loop {
				while let Some(_) = quit_signal_consumer.pop() {
					break;
				}
			}
		});
		Ok(Self {
			quit_signal_producer,
			command_producer,
			event_consumer,
			sounds_to_unload_consumer,
			sequences_to_unload_consumer,
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

	/// Starts or resumes the metronome.
	pub fn start_metronome(&mut self) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Metronome(MetronomeCommand::StartMetronome))
		{
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Pauses the metronome.
	pub fn pause_metronome(&mut self) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Metronome(MetronomeCommand::PauseMetronome))
		{
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Stops and resets the metronome.
	pub fn stop_metronome(&mut self) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::Metronome(MetronomeCommand::StopMetronome))
		{
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Starts a sequence.
	pub fn start_sequence(&mut self, sequence: Sequence) -> Result<SequenceId, ConductorError> {
		let id = SequenceId::new();
		match self
			.command_producer
			.push(Command::Sequence(SequenceCommand::StartSequence(
				id, sequence,
			))) {
			Ok(_) => Ok(id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Returns a list of all of the new events created by the audio thread
	/// (since the last time `events` was called).
	pub fn events(&mut self) -> Vec<Event> {
		let mut events = vec![];
		while let Some(event) = self.event_consumer.pop() {
			events.push(event);
		}
		events
	}

	/// Frees resources that are no longer in use, such as unloaded sounds
	/// or finished sequences.
	pub fn free_unused_resources(&mut self) {
		while let Some(_) = self.sounds_to_unload_consumer.pop() {}
		while let Some(_) = self.sequences_to_unload_consumer.pop() {}
	}
}

impl Drop for AudioManager {
	fn drop(&mut self) {
		self.quit_signal_producer.push(true).unwrap();
	}
}
