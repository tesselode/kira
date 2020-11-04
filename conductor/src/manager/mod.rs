pub(crate) mod backend;

use crate::{
	command::MixerCommand,
	command::ParameterCommand,
	command::{Command, InstanceCommand, MetronomeCommand, SequenceCommand, SoundCommand},
	error::ConductorError,
	error::ConductorResult,
	instance::{InstanceId, InstanceSettings},
	metronome::MetronomeSettings,
	parameter::ParameterId,
	sequence::{Sequence, SequenceId},
	sound::{Sound, SoundId, SoundSettings},
	tempo::Tempo,
	track::effect::Effect,
	track::effect::EffectId,
	track::effect_slot::EffectSlot,
	track::id::SubTrackId,
	track::index::TrackIndex,
	track::EffectSettings,
	track::Track,
	track::TrackSettings,
	tween::Tween,
	value::Value,
};
use backend::Backend;
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::path::Path;

const WRAPPER_THREAD_SLEEP_DURATION: f64 = 1.0 / 60.0;

/// Events that can be sent by the audio thread.
#[derive(Debug, Copy, Clone)]
pub enum Event<CustomEvent: Send + 'static> {
	/**
	Sent when the metronome passes a certain interval (in beats).

	For example, an event with an interval of `1.0` will be sent
	every beat, and an event with an interval of `0.25` will be
	sent every sixteenth note (one quarter of a beat).

	The intervals that a metronome emits events for are defined
	when the metronome is created.
	*/
	MetronomeIntervalPassed(f64),
	Custom(CustomEvent),
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
	/// The maximum number of parameters that can exist at once.
	pub num_parameters: usize,
	/// The maximum number of instances of sounds that can be playing at once.
	pub num_instances: usize,
	/// The maximum number of sequences that can be running at a time.
	pub num_sequences: usize,
	/// The maximum number of mixer tracks that can be used at a time.
	pub num_tracks: usize,
	/// The maximum number of effects that can be running at a time on a track.
	pub num_effects_per_track: usize,
	/// Settings for the metronome.
	pub metronome_settings: MetronomeSettings,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_events: 100,
			num_sounds: 100,
			num_parameters: 100,
			num_instances: 100,
			num_sequences: 25,
			num_tracks: 100,
			num_effects_per_track: 10,
			metronome_settings: MetronomeSettings::default(),
		}
	}
}

impl From<()> for AudioManagerSettings {
	fn from(_: ()) -> Self {
		Self::default()
	}
}

/**
Plays and manages audio.

The `AudioManager` is responsible for all communication between the gameplay thread
and the audio thread.
*/
pub struct AudioManager<CustomEvent: Send + 'static = ()> {
	quit_signal_producer: Producer<bool>,
	command_producer: Producer<Command<CustomEvent>>,
	event_consumer: Consumer<Event<CustomEvent>>,
	sounds_to_unload_consumer: Consumer<Sound>,
	sequences_to_unload_consumer: Consumer<Sequence<CustomEvent>>,
	tracks_to_unload_consumer: Consumer<Track>,
	effect_slots_to_unload_consumer: Consumer<EffectSlot>,
}

impl<CustomEvent: Copy + Send + 'static + std::fmt::Debug> AudioManager<CustomEvent> {
	/// Creates a new audio manager and starts an audio thread.
	pub fn new(settings: AudioManagerSettings) -> ConductorResult<Self> {
		// set up various ringbuffers for communication between threads
		let (quit_signal_producer, mut quit_signal_consumer) = RingBuffer::new(1).split();
		let (mut setup_result_producer, mut setup_result_consumer) =
			RingBuffer::<ConductorResult<()>>::new(1).split();
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let (sounds_to_unload_producer, sounds_to_unload_consumer) =
			RingBuffer::new(settings.num_sounds).split();
		let (sequences_to_unload_producer, sequences_to_unload_consumer) =
			RingBuffer::new(settings.num_sequences).split();
		let (tracks_to_unload_producer, tracks_to_unload_consumer) =
			RingBuffer::new(settings.num_tracks).split();
		let (effect_slots_to_unload_producer, effect_slots_to_unload_consumer) =
			RingBuffer::new(settings.num_tracks * settings.num_effects_per_track).split();
		let (event_producer, event_consumer) = RingBuffer::new(settings.num_events).split();
		// set up a cpal stream on a new thread. we could do this on the main thread,
		// but that causes issues with LÖVE.
		std::thread::spawn(move || {
			let setup_result = || -> ConductorResult<Stream> {
				let host = cpal::default_host();
				let device = match host.default_output_device() {
					Some(device) => device,
					None => return Err(ConductorError::NoDefaultOutputDevice),
				};
				let config = match device.supported_output_configs()?.next() {
					Some(config) => config,
					None => return Err(ConductorError::NoSupportedAudioConfig),
				}
				.with_max_sample_rate()
				.config();
				let sample_rate = config.sample_rate.0;
				let channels = config.channels;
				let mut backend = Backend::new(
					sample_rate,
					settings,
					command_consumer,
					event_producer,
					sounds_to_unload_producer,
					sequences_to_unload_producer,
					tracks_to_unload_producer,
					effect_slots_to_unload_producer,
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
				Ok(stream)
			}();
			match setup_result {
				Ok(_stream) => {
					setup_result_producer.push(Ok(())).unwrap();
					// wait for a quit message before ending the thread and dropping
					// the stream
					while let None = quit_signal_consumer.pop() {
						std::thread::sleep(std::time::Duration::from_secs_f64(
							WRAPPER_THREAD_SLEEP_DURATION,
						));
					}
				}
				Err(error) => {
					setup_result_producer.push(Err(error)).unwrap();
				}
			}
		});
		// wait for the audio thread to report back a result
		loop {
			if let Some(result) = setup_result_consumer.pop() {
				match result {
					Ok(_) => break,
					Err(error) => return Err(error),
				}
			}
		}
		Ok(Self {
			quit_signal_producer,
			command_producer,
			event_consumer,
			sounds_to_unload_consumer,
			sequences_to_unload_consumer,
			tracks_to_unload_consumer,
			effect_slots_to_unload_consumer,
		})
	}

	fn send_command_to_backend(&mut self, command: Command<CustomEvent>) -> ConductorResult<()> {
		match self.command_producer.push(command) {
			Ok(_) => Ok(()),
			Err(_) => Err(ConductorError::CommandQueueFull),
		}
	}

	pub fn add_parameter(&mut self, value: f64) -> ConductorResult<ParameterId> {
		let id = ParameterId::new();
		self.send_command_to_backend(Command::Parameter(ParameterCommand::AddParameter(
			id, value,
		)))?;
		Ok(id)
	}

	pub fn remove_parameter(&mut self, id: ParameterId) -> ConductorResult<()> {
		self.send_command_to_backend(Command::Parameter(ParameterCommand::RemoveParameter(id)))
	}

	pub fn set_parameter(
		&mut self,
		id: ParameterId,
		value: f64,
		tween: Option<Tween>,
	) -> ConductorResult<()> {
		self.send_command_to_backend(Command::Parameter(ParameterCommand::SetParameter(
			id, value, tween,
		)))
	}

	pub fn add_sub_track(&mut self, settings: TrackSettings) -> ConductorResult<SubTrackId> {
		let id = SubTrackId::new();
		self.send_command_to_backend(Command::Mixer(MixerCommand::AddSubTrack(id, settings)))?;
		Ok(id)
	}

	pub fn remove_sub_track(&mut self, id: SubTrackId) -> ConductorResult<()> {
		self.send_command_to_backend(Command::Mixer(MixerCommand::RemoveSubTrack(id)))
	}

	pub fn add_effect_to_track(
		&mut self,
		track_index: TrackIndex,
		effect: Box<dyn Effect>,
		settings: EffectSettings,
	) -> ConductorResult<EffectId> {
		let effect_id = EffectId::new(track_index);
		self.send_command_to_backend(Command::Mixer(MixerCommand::AddEffect(
			track_index,
			effect_id,
			effect,
			settings,
		)))?;
		Ok(effect_id)
	}

	pub fn remove_effect(&mut self, effect_id: EffectId) -> ConductorResult<()> {
		self.send_command_to_backend(Command::Mixer(MixerCommand::RemoveEffect(effect_id)))
	}

	/// Loads a sound from a file path.
	///
	/// Returns a handle to the sound. Keep this so you can play the sound later.
	pub fn load_sound<P, S: Into<SoundSettings>>(
		&mut self,
		path: P,
		settings: S,
	) -> ConductorResult<SoundId>
	where
		P: AsRef<Path>,
	{
		let settings = settings.into();
		let sound = Sound::from_file(path, &settings)?;
		let id = SoundId::new(sound.duration(), settings.default_track, settings.metadata);
		self.send_command_to_backend(Command::Sound(SoundCommand::LoadSound(id, sound)))?;
		Ok(id)
	}

	/// Unloads a sound, deallocating its memory.
	pub fn unload_sound(&mut self, id: SoundId) -> ConductorResult<()> {
		self.send_command_to_backend(Command::Sound(SoundCommand::UnloadSound(id)))
	}

	/// Plays a sound.
	pub fn play_sound<S: Into<InstanceSettings>>(
		&mut self,
		sound_id: SoundId,
		settings: S,
	) -> Result<InstanceId, ConductorError> {
		let instance_id = InstanceId::new();
		self.send_command_to_backend(Command::Instance(InstanceCommand::PlaySound(
			instance_id,
			sound_id,
			None,
			settings.into(),
		)))?;
		Ok(instance_id)
	}

	pub fn set_instance_volume(
		&mut self,
		id: InstanceId,
		volume: Value,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::SetInstanceVolume(
			id, volume,
		)))
	}

	pub fn set_instance_pitch(
		&mut self,
		id: InstanceId,
		pitch: Value,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::SetInstancePitch(
			id, pitch,
		)))
	}

	/// Pauses a currently playing instance of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn pause_instance(
		&mut self,
		instance_id: InstanceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::PauseInstance(
			instance_id,
			fade_tween,
		)))
	}

	/// Resumes a currently paused instance of a sound.
	///
	/// You can optionally provide a fade-in duration (in seconds).
	pub fn resume_instance(
		&mut self,
		instance_id: InstanceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::ResumeInstance(
			instance_id,
			fade_tween,
		)))
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
		self.send_command_to_backend(Command::Instance(InstanceCommand::StopInstance(
			instance_id,
			fade_tween,
		)))
	}

	/// Pauses all currently playing instances of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn pause_instances_of_sound(
		&mut self,
		sound_id: SoundId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::PauseInstancesOfSound(
			sound_id, fade_tween,
		)))
	}

	/// Resumes all currently playing instances of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn resume_instances_of_sound(
		&mut self,
		sound_id: SoundId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::ResumeInstancesOfSound(
			sound_id, fade_tween,
		)))
	}

	/// Stops all currently playing instances of a sound.
	///
	/// You can optionally provide a fade-out duration (in seconds).
	pub fn stop_instances_of_sound(
		&mut self,
		sound_id: SoundId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Instance(InstanceCommand::StopInstancesOfSound(
			sound_id, fade_tween,
		)))
	}

	/// Sets the tempo of the metronome.
	pub fn set_metronome_tempo(&mut self, tempo: Tempo) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Metronome(MetronomeCommand::SetMetronomeTempo(
			tempo,
		)))
	}

	/// Starts or resumes the metronome.
	pub fn start_metronome(&mut self) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Metronome(MetronomeCommand::StartMetronome))
	}

	/// Pauses the metronome.
	pub fn pause_metronome(&mut self) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Metronome(MetronomeCommand::PauseMetronome))
	}

	/// Stops and resets the metronome.
	pub fn stop_metronome(&mut self) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Metronome(MetronomeCommand::StopMetronome))
	}

	/// Starts a sequence.
	pub fn start_sequence(
		&mut self,
		sequence: Sequence<CustomEvent>,
	) -> Result<SequenceId, ConductorError> {
		sequence.validate()?;
		let id = SequenceId::new();
		self.send_command_to_backend(Command::Sequence(SequenceCommand::StartSequence(
			id, sequence,
		)))?;
		Ok(id)
	}

	/// Mutes a sequence.
	///
	/// When a sequence is muted, it will continue waiting for durations and intervals,
	/// but it will not change anything that changes the audio, like starting new sounds
	/// or settings the metronome tempo.
	pub fn mute_sequence(&mut self, id: SequenceId) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::MuteSequence(id)))
	}

	/// Unmutes a sequence.
	pub fn unmute_sequence(&mut self, id: SequenceId) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::UnmuteSequence(id)))
	}

	/// Pauses a sequence.
	pub fn pause_sequence(&mut self, id: SequenceId) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::PauseSequence(id)))
	}

	/// Resumes a sequence.
	pub fn resume_sequence(&mut self, id: SequenceId) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::ResumeSequence(id)))
	}

	/// Stops a sequence.
	pub fn stop_sequence(&mut self, id: SequenceId) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::StopSequence(id)))
	}

	/// Pauses a sequence and any instances played by that sequence.
	pub fn pause_sequence_and_instances(
		&mut self,
		id: SequenceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::PauseSequence(id)))?;
		self.send_command_to_backend(Command::Instance(
			InstanceCommand::PauseInstancesOfSequence(id, fade_tween),
		))
	}

	/// Resumes a sequence and any instances played by that sequence.
	pub fn resume_sequence_and_instances(
		&mut self,
		id: SequenceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::ResumeSequence(id)))?;
		self.send_command_to_backend(Command::Instance(
			InstanceCommand::ResumeInstancesOfSequence(id, fade_tween),
		))
	}

	/// Stops a sequence and any instances played by that sequence.
	pub fn stop_sequence_and_instances(
		&mut self,
		id: SequenceId,
		fade_tween: Option<Tween>,
	) -> Result<(), ConductorError> {
		self.send_command_to_backend(Command::Sequence(SequenceCommand::StopSequence(id)))?;
		self.send_command_to_backend(Command::Instance(InstanceCommand::StopInstancesOfSequence(
			id, fade_tween,
		)))
	}

	/// Returns a list of all of the new events created by the audio thread
	/// (since the last time `events` was called).
	pub fn events(&mut self) -> Vec<Event<CustomEvent>> {
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
		while let Some(_) = self.tracks_to_unload_consumer.pop() {}
		while let Some(_) = self.effect_slots_to_unload_consumer.pop() {}
	}
}

impl<CustomEvent: Send + 'static> Drop for AudioManager<CustomEvent> {
	fn drop(&mut self) {
		self.quit_signal_producer.push(true).unwrap();
	}
}
