mod backend;

use crate::{
	command::Command,
	error::ConductorError,
	instance::{InstanceId, InstanceSettings},
	metronome::MetronomeId,
	project::Project,
	sequence::{Sequence, SequenceId},
	sound::SoundId,
	tween::Tween,
};
use backend::Backend;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::error::Error;

/// Events that can be sent by the audio thread.
#[derive(Debug)]
pub enum Event {
	/**
	Sent when a metronome passes a certain interval (in beats).

	For example, an event with an interval of `1.0` will be sent
	every beat, and an event with an interval of `0.25` will be
	sent every sixteenth note (one quarter of a beat).

	The intervals that a metronome emits events for are defined
	when the metronome is created.
	*/
	MetronomeIntervalPassed(MetronomeId, f32),
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
	/// The maximum number of instances of sounds that can be playing
	/// at a time.
	pub num_instances: usize,
	/// The maximum number of sequences that can be running at a time.
	pub num_sequences: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_events: 100,
			num_instances: 100,
			num_sequences: 50,
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
	event_consumer: Consumer<Event>,
	_stream: Stream,
}

impl AudioManager {
	/// Creates a new audio manager and starts an audio thread.
	///
	/// The `Project` is given to the audio thread, so make sure you've loaded all
	/// the sounds you want to use before you create an `AudioManager`.
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
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let (event_producer, event_consumer) = RingBuffer::new(settings.num_events).split();
		let mut backend = Backend::new(
			sample_rate,
			project,
			command_consumer,
			event_producer,
			settings,
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

	/// Plays a sound.
	pub fn play_sound(
		&mut self,
		sound_id: SoundId,
		settings: InstanceSettings,
	) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self
			.command_producer
			.push(Command::PlaySound(sound_id, instance_id, settings))
		{
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
			.push(Command::SetInstanceVolume(id, volume, tween))
		{
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
			.push(Command::SetInstancePitch(id, pitch, tween))
		{
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
		fade_duration: Option<f32>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::PauseInstance(instance_id, fade_duration))
		{
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
		fade_duration: Option<f32>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::ResumeInstance(instance_id, fade_duration))
		{
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
		fade_duration: Option<f32>,
	) -> Result<(), ConductorError> {
		match self
			.command_producer
			.push(Command::StopInstance(instance_id, fade_duration))
		{
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Starts or resumes a metronome.
	pub fn start_metronome(&mut self, id: MetronomeId) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self.command_producer.push(Command::StartMetronome(id)) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Pauses a metronome.
	pub fn pause_metronome(&mut self, id: MetronomeId) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self.command_producer.push(Command::PauseMetronome(id)) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Stops and resets a metronome.
	pub fn stop_metronome(&mut self, id: MetronomeId) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		match self.command_producer.push(Command::StopMetronome(id)) {
			Ok(_) => Ok(instance_id),
			Err(_) => Err(ConductorError::SendCommand),
		}
	}

	/// Starts a sequence.
	pub fn start_sequence(&mut self, sequence: Sequence) -> Result<SequenceId, ConductorError> {
		let id = SequenceId::new();
		match self
			.command_producer
			.push(Command::StartSequence(id, sequence))
		{
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
}
