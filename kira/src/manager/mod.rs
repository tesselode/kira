//! Provides a bridge between the main thread and the audio thread.

mod backend;

use std::{hash::Hash, path::Path};

#[cfg(not(feature = "benchmarking"))]
use backend::Backend;
#[cfg(feature = "benchmarking")]
pub use backend::Backend;

use crate::{
	arrangement::{Arrangement, ArrangementId},
	audio_stream::{AudioStream, AudioStreamId},
	command::{
		Command, GroupCommand, InstanceCommand, MetronomeCommand, MixerCommand, ParameterCommand,
		ResourceCommand, SequenceCommand, StreamCommand,
	},
	error::{AudioError, AudioResult},
	group::{Group, GroupId},
	instance::{
		InstanceId, InstanceSettings, PauseInstanceSettings, ResumeInstanceSettings,
		StopInstanceSettings,
	},
	metronome::MetronomeSettings,
	mixer::{
		effect::{Effect, EffectId, EffectSettings},
		effect_slot::EffectSlot,
		SubTrackId, Track, TrackIndex, TrackSettings,
	},
	parameter::{ParameterId, Tween},
	playable::{Playable, PlayableSettings},
	sequence::SequenceInstance,
	sequence::{EventReceiver, Sequence, SequenceInstanceId, SequenceInstanceSettings},
	sound::{Sound, SoundId},
	tempo::Tempo,
	util::index_set_from_vec,
	value::Value,
	Event,
};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Consumer, Producer, RingBuffer};

use self::backend::BackendThreadChannels;

const WRAPPER_THREAD_SLEEP_DURATION: f64 = 1.0 / 60.0;

/// Settings for an [`AudioManager`](crate::manager::AudioManager).
#[derive(Debug, Clone)]
pub struct AudioManagerSettings {
	/// The number of commands that be sent to the audio thread at a time.
	///
	/// Each action you take, like starting an instance or pausing a sequence,
	/// queues up one command.
	pub num_commands: usize,
	/// The number of events the audio thread can send at a time.
	pub num_events: usize,
	/// The maximum number of sounds that can be loaded at a time.
	pub num_sounds: usize,
	/// The maximum number of arrangements that can be loaded at a time.
	pub num_arrangements: usize,
	/// The maximum number of parameters that can exist at a time.
	pub num_parameters: usize,
	/// The maximum number of instances of sounds that can be playing at a time.
	pub num_instances: usize,
	/// The maximum number of sequences that can be running at a time.
	pub num_sequences: usize,
	/// The maximum number of mixer tracks that can be used at a time.
	pub num_tracks: usize,
	/// The maximum number of effects that can be running at a time on a mixer track.
	pub num_effects_per_track: usize,
	/// The maximum number of groups that can be used at a time.
	pub num_groups: usize,
	/// The maximum number of audio strams that can be used at a time.
	pub num_streams: usize,
	/// Settings for the metronome.
	pub metronome_settings: MetronomeSettings,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_events: 100,
			num_sounds: 100,
			num_arrangements: 100,
			num_parameters: 100,
			num_instances: 100,
			num_sequences: 25,
			num_tracks: 100,
			num_effects_per_track: 10,
			num_groups: 100,
			num_streams: 100,
			metronome_settings: MetronomeSettings::default(),
		}
	}
}

pub(crate) struct AudioManagerThreadChannels {
	pub quit_signal_producer: Producer<bool>,
	pub command_producer: Producer<Command>,
	pub event_consumer: Consumer<Event>,
	pub sounds_to_unload_consumer: Consumer<Sound>,
	pub arrangements_to_unload_consumer: Consumer<Arrangement>,
	pub sequence_instances_to_unload_consumer: Consumer<SequenceInstance>,
	pub tracks_to_unload_consumer: Consumer<Track>,
	pub effect_slots_to_unload_consumer: Consumer<EffectSlot>,
	pub groups_to_unload_consumer: Consumer<Group>,
	pub streams_to_unload_consumer: Consumer<Box<dyn AudioStream>>,
}

/**
Plays and manages audio.

The audio manager is responsible for all communication between the gameplay thread
and the audio thread.
*/
pub struct AudioManager {
	thread_channels: AudioManagerThreadChannels,
	// holds the stream if it has been created on the main thread
	// so it can live for as long as the audio manager
	_stream: Option<Stream>,
}

impl AudioManager {
	/// Creates a new audio manager and starts an audio thread.
	pub fn new(settings: AudioManagerSettings) -> AudioResult<Self> {
		let (audio_manager_thread_channels, backend_thread_channels, mut quit_signal_consumer) =
			Self::create_thread_channels(&settings);

		let stream = {
			let (mut setup_result_producer, mut setup_result_consumer) =
				RingBuffer::<AudioResult<()>>::new(1).split();
			// set up a cpal stream on a new thread. we could do this on the main thread,
			// but that causes issues with LÖVE.
			std::thread::spawn(move || {
				match Self::setup_stream(settings, backend_thread_channels) {
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

			None
		};

		Ok(Self {
			thread_channels: audio_manager_thread_channels,
			_stream: stream,
		})
	}

	fn create_thread_channels(
		settings: &AudioManagerSettings,
	) -> (
		AudioManagerThreadChannels,
		BackendThreadChannels,
		Consumer<bool>,
	) {
		let (quit_signal_producer, quit_signal_consumer) = RingBuffer::new(1).split();
		let (command_producer, command_consumer) = RingBuffer::new(settings.num_commands).split();
		let (sounds_to_unload_producer, sounds_to_unload_consumer) =
			RingBuffer::new(settings.num_sounds).split();
		let (arrangements_to_unload_producer, arrangements_to_unload_consumer) =
			RingBuffer::new(settings.num_arrangements).split();
		let (sequence_instances_to_unload_producer, sequence_instances_to_unload_consumer) =
			RingBuffer::new(settings.num_sequences).split();
		let (tracks_to_unload_producer, tracks_to_unload_consumer) =
			RingBuffer::new(settings.num_tracks).split();
		let (effect_slots_to_unload_producer, effect_slots_to_unload_consumer) =
			RingBuffer::new(settings.num_tracks * settings.num_effects_per_track).split();
		let (groups_to_unload_producer, groups_to_unload_consumer) =
			RingBuffer::new(settings.num_groups).split();
		let (event_producer, event_consumer) = RingBuffer::new(settings.num_events).split();
		let (streams_to_unload_producer, streams_to_unload_consumer) =
			RingBuffer::new(settings.num_streams).split();
		let audio_manager_thread_channels = AudioManagerThreadChannels {
			quit_signal_producer,
			command_producer,
			event_consumer,
			sounds_to_unload_consumer,
			arrangements_to_unload_consumer,
			sequence_instances_to_unload_consumer,
			tracks_to_unload_consumer,
			effect_slots_to_unload_consumer,
			groups_to_unload_consumer,
			streams_to_unload_consumer,
		};
		let backend_thread_channels = BackendThreadChannels {
			command_consumer,
			event_producer,
			sounds_to_unload_producer,
			arrangements_to_unload_producer,
			sequence_instances_to_unload_producer,
			tracks_to_unload_producer,
			effect_slots_to_unload_producer,
			groups_to_unload_producer,
			streams_to_unload_producer,
		};
		(
			audio_manager_thread_channels,
			backend_thread_channels,
			quit_signal_consumer,
		)
	}

	fn setup_stream(
		settings: AudioManagerSettings,
		backend_thread_channels: BackendThreadChannels,
	) -> AudioResult<Stream> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(AudioError::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let mut backend = Backend::new(sample_rate, settings, backend_thread_channels);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					if channels == 1 {
						frame[0] = (out.left + out.right) / 2.0;
					} else {
						frame[0] = out.left;
						frame[1] = out.right;
					}
				}
			},
			move |_| {},
		)?;
		stream.play()?;
		Ok(stream)
	}

	#[cfg(feature = "benchmarking")]
	pub fn new_without_audio_thread(
		settings: AudioManagerSettings,
	) -> AudioResult<(Self, Backend)> {
		const SAMPLE_RATE: u32 = 48000;
		let (audio_manager_thread_channels, backend_thread_channels, _) =
			Self::create_thread_channels(&settings);
		let audio_manager = Self {
			thread_channels: audio_manager_thread_channels,
			_stream: None,
		};
		let backend = Backend::new(SAMPLE_RATE, settings, backend_thread_channels);
		Ok((audio_manager, backend))
	}

	fn send_command_to_backend<C: Into<Command>>(&mut self, command: C) -> AudioResult<()> {
		match self.thread_channels.command_producer.push(command.into()) {
			Ok(_) => Ok(()),
			Err(_) => Err(AudioError::CommandQueueFull),
		}
	}

	/// Sends a sound to the audio thread and returns a handle to the sound.
	pub fn add_sound(&mut self, sound: Sound) -> AudioResult<SoundId> {
		let id = SoundId::new(&sound);
		self.send_command_to_backend(ResourceCommand::AddSound(id, sound))?;
		Ok(id)
	}

	/// Loads a sound from a file and returns a handle to the sound.
	///
	/// This is a shortcut for constructing the sound manually and adding it
	/// using [`AudioManager::add_sound`].
	pub fn load_sound<P: AsRef<Path>>(
		&mut self,
		path: P,
		settings: PlayableSettings,
	) -> AudioResult<SoundId> {
		let sound = Sound::from_file(path, settings)?;
		self.add_sound(sound)
	}

	/// Removes a sound from the audio thread, allowing its memory to be freed.
	pub fn remove_sound(&mut self, id: SoundId) -> AudioResult<()> {
		self.send_command_to_backend(ResourceCommand::RemoveSound(id))
	}

	/// Sends a arrangement to the audio thread and returns a handle to the arrangement.
	pub fn add_arrangement(&mut self, arrangement: Arrangement) -> AudioResult<ArrangementId> {
		let id = ArrangementId::new(&arrangement);
		self.send_command_to_backend(ResourceCommand::AddArrangement(id, arrangement))?;
		Ok(id)
	}

	/// Removes a arrangement from the audio thread, allowing its memory to be freed.
	pub fn remove_arrangement(&mut self, id: ArrangementId) -> AudioResult<()> {
		self.send_command_to_backend(ResourceCommand::RemoveArrangement(id))
	}

	/// Frees resources that are no longer in use, such as unloaded sounds
	/// or finished sequences.
	pub fn free_unused_resources(&mut self) {
		while let Some(_) = self.thread_channels.sounds_to_unload_consumer.pop() {}
		while let Some(_) = self.thread_channels.arrangements_to_unload_consumer.pop() {}
		while let Some(_) = self
			.thread_channels
			.sequence_instances_to_unload_consumer
			.pop()
		{}
		while let Some(_) = self.thread_channels.tracks_to_unload_consumer.pop() {}
		while let Some(_) = self.thread_channels.effect_slots_to_unload_consumer.pop() {}
		while let Some(_) = self.thread_channels.groups_to_unload_consumer.pop() {}
		while let Some(_) = self.thread_channels.streams_to_unload_consumer.pop() {}
	}

	/// Plays a sound or arrangement.
	pub fn play<P: Into<Playable>>(
		&mut self,
		playable: P,
		settings: InstanceSettings,
	) -> Result<InstanceId, AudioError> {
		let instance_id = InstanceId::new();
		self.send_command_to_backend(InstanceCommand::Play(
			instance_id,
			playable.into(),
			None,
			settings,
		))?;
		Ok(instance_id)
	}

	/// Sets the volume of an instance.
	pub fn set_instance_volume<V: Into<Value<f64>>>(
		&mut self,
		id: InstanceId,
		volume: V,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::SetInstanceVolume(id, volume.into()))
	}

	/// Sets the pitch of an instance.
	pub fn set_instance_pitch<V: Into<Value<f64>>>(
		&mut self,
		id: InstanceId,
		pitch: V,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::SetInstancePitch(id, pitch.into()))
	}

	/// Sets the panning of an instance (0 = hard left, 1 = hard right).
	pub fn set_instance_panning<V: Into<Value<f64>>>(
		&mut self,
		id: InstanceId,
		panning: V,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::SetInstancePanning(id, panning.into()))
	}

	/// Moves the playback position of an instance backward or forward.
	pub fn seek_instance(&mut self, id: InstanceId, offset: f64) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::SeekInstance(id, offset))
	}

	/// Sets the playback position of an instance.
	pub fn seek_instance_to(&mut self, id: InstanceId, position: f64) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::SeekInstanceTo(id, position))
	}

	/// Pauses a currently playing instance of a sound with an optional fade-out tween.
	pub fn pause_instance(
		&mut self,
		instance_id: InstanceId,
		settings: PauseInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::PauseInstance(instance_id, settings))
	}

	/// Resumes a currently paused instance of a sound with an optional fade-in tween.
	pub fn resume_instance(
		&mut self,
		instance_id: InstanceId,
		settings: ResumeInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::ResumeInstance(instance_id, settings))
	}

	/// Stops a currently playing instance of a sound with an optional fade-out tween.
	///
	/// Once the instance is stopped, it cannot be restarted.
	pub fn stop_instance(
		&mut self,
		instance_id: InstanceId,
		settings: StopInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::StopInstance(instance_id, settings))
	}

	/// Pauses all currently playing instances of a sound or arrangement with an optional fade-out tween.
	pub fn pause_instances_of(
		&mut self,
		playable: Playable,
		settings: PauseInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::PauseInstancesOf(playable, settings))
	}

	/// Resumes all currently playing instances of a sound or arrangement with an optional fade-in tween.
	pub fn resume_instances_of(
		&mut self,
		playable: Playable,
		settings: ResumeInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::ResumeInstancesOf(playable, settings))
	}

	/// Stops all currently playing instances of a sound or arrangement with an optional fade-out tween.
	pub fn stop_instances_of(
		&mut self,
		playable: Playable,
		settings: StopInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(InstanceCommand::StopInstancesOf(playable, settings))
	}

	/// Sets the tempo of the metronome.
	pub fn set_metronome_tempo<T: Into<Value<Tempo>>>(
		&mut self,
		tempo: T,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(MetronomeCommand::SetMetronomeTempo(tempo.into()))
	}

	/// Starts or resumes the metronome.
	pub fn start_metronome(&mut self) -> Result<(), AudioError> {
		self.send_command_to_backend(MetronomeCommand::StartMetronome)
	}

	/// Pauses the metronome.
	pub fn pause_metronome(&mut self) -> Result<(), AudioError> {
		self.send_command_to_backend(MetronomeCommand::PauseMetronome)
	}

	/// Stops and resets the metronome.
	pub fn stop_metronome(&mut self) -> Result<(), AudioError> {
		self.send_command_to_backend(MetronomeCommand::StopMetronome)
	}

	/// Starts a sequence.
	pub fn start_sequence<CustomEvent: Clone + Eq + Hash>(
		&mut self,
		sequence: Sequence<CustomEvent>,
		settings: SequenceInstanceSettings,
	) -> Result<(SequenceInstanceId, EventReceiver<CustomEvent>), AudioError> {
		sequence.validate()?;
		let id = SequenceInstanceId::new();
		let (instance, receiver) = sequence.create_instance(settings);
		self.send_command_to_backend(SequenceCommand::StartSequenceInstance(id, instance))?;
		Ok((id, receiver))
	}

	/// Mutes a sequence.
	///
	/// When a sequence is muted, it will continue waiting for durations and intervals,
	/// but it will not play sounds, emit events, or perform any other actions.
	pub fn mute_sequence(&mut self, id: SequenceInstanceId) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::MuteSequenceInstance(id))
	}

	/// Unmutes a sequence.
	pub fn unmute_sequence(&mut self, id: SequenceInstanceId) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::UnmuteSequenceInstance(id))
	}

	/// Pauses a sequence.
	pub fn pause_sequence(&mut self, id: SequenceInstanceId) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::PauseSequenceInstance(id))
	}

	/// Resumes a sequence.
	pub fn resume_sequence(&mut self, id: SequenceInstanceId) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::ResumeSequenceInstance(id))
	}

	/// Stops a sequence.
	pub fn stop_sequence(&mut self, id: SequenceInstanceId) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::StopSequenceInstance(id))
	}

	/// Pauses a sequence and any instances played by that sequence.
	pub fn pause_sequence_and_instances(
		&mut self,
		id: SequenceInstanceId,
		settings: PauseInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::PauseSequenceInstance(id))?;
		self.send_command_to_backend(InstanceCommand::PauseInstancesOfSequence(id, settings))
	}

	/// Resumes a sequence and any instances played by that sequence.
	pub fn resume_sequence_and_instances(
		&mut self,
		id: SequenceInstanceId,
		settings: ResumeInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::ResumeSequenceInstance(id))?;
		self.send_command_to_backend(InstanceCommand::ResumeInstancesOfSequence(id, settings))
	}

	/// Stops a sequence and any instances played by that sequence.
	pub fn stop_sequence_and_instances(
		&mut self,
		id: SequenceInstanceId,
		settings: StopInstanceSettings,
	) -> Result<(), AudioError> {
		self.send_command_to_backend(SequenceCommand::StopSequenceInstance(id))?;
		self.send_command_to_backend(InstanceCommand::StopInstancesOfSequence(id, settings))
	}

	/// Creates a parameter with the specified starting value.
	pub fn add_parameter(&mut self, value: f64) -> AudioResult<ParameterId> {
		let id = ParameterId::new();
		self.send_command_to_backend(ParameterCommand::AddParameter(id, value))?;
		Ok(id)
	}

	/// Removes a parameter.
	pub fn remove_parameter(&mut self, id: ParameterId) -> AudioResult<()> {
		self.send_command_to_backend(ParameterCommand::RemoveParameter(id))
	}

	/// Sets the value of a parameter with an optional tween to smoothly change the value.
	pub fn set_parameter(
		&mut self,
		id: ParameterId,
		value: f64,
		tween: Option<Tween>,
	) -> AudioResult<()> {
		self.send_command_to_backend(ParameterCommand::SetParameter(id, value, tween))
	}

	/// Creates a mixer sub-track.
	pub fn add_sub_track(&mut self, settings: TrackSettings) -> AudioResult<SubTrackId> {
		let id = SubTrackId::new();
		self.send_command_to_backend(MixerCommand::AddSubTrack(id, Track::new(settings)))?;
		Ok(id)
	}

	/// Removes a sub-track from the mixer.
	pub fn remove_sub_track(&mut self, id: SubTrackId) -> AudioResult<()> {
		self.send_command_to_backend(MixerCommand::RemoveSubTrack(id))
	}

	/// Adds an effect to a track.
	pub fn add_effect_to_track<T: Into<TrackIndex> + Copy, E: Effect + 'static>(
		&mut self,
		track_index: T,
		effect: E,
		settings: EffectSettings,
	) -> AudioResult<EffectId> {
		let effect_id = EffectId::new(track_index.into());
		self.send_command_to_backend(MixerCommand::AddEffect(
			track_index.into(),
			effect_id,
			Box::new(effect),
			settings,
		))?;
		Ok(effect_id)
	}

	/// Removes an effect from the mixer.
	pub fn remove_effect(&mut self, effect_id: EffectId) -> AudioResult<()> {
		self.send_command_to_backend(MixerCommand::RemoveEffect(effect_id))
	}

	/// Starts an audio stream on the specified track.
	pub fn add_stream<T: Into<TrackIndex>, S: AudioStream>(
		&mut self,
		track_index: T,
		stream: S,
	) -> AudioResult<AudioStreamId> {
		let stream_id = AudioStreamId::new();
		self.send_command_to_backend(StreamCommand::AddStream(
			stream_id,
			track_index.into(),
			Box::new(stream),
		))
		.map(|()| stream_id)
	}

	/// Stops and drops the specified audio stream.
	pub fn remove_stream(&mut self, stream_id: AudioStreamId) -> AudioResult<()> {
		self.send_command_to_backend(StreamCommand::RemoveStream(stream_id))
	}

	/// Adds a group.
	pub fn add_group<T: Into<Vec<GroupId>>>(&mut self, parent_groups: T) -> AudioResult<GroupId> {
		let id = GroupId::new();
		let group = Group::new(index_set_from_vec(parent_groups.into()));
		self.send_command_to_backend(GroupCommand::AddGroup(id, group))?;
		Ok(id)
	}

	/// Removes a group.
	pub fn remove_group(&mut self, id: GroupId) -> AudioResult<()> {
		self.send_command_to_backend(GroupCommand::RemoveGroup(id))
	}

	/// Pauses all instances of sounds, arrangements, and sequences in a group.
	pub fn pause_group(&mut self, id: GroupId, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::PauseGroup(id, settings))?;
		self.send_command_to_backend(SequenceCommand::PauseGroup(id))?;
		Ok(())
	}

	/// Resumes all instances of sounds, arrangements, and sequences in a group.
	pub fn resume_group(
		&mut self,
		id: GroupId,
		settings: ResumeInstanceSettings,
	) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::ResumeGroup(id, settings))?;
		self.send_command_to_backend(SequenceCommand::ResumeGroup(id))?;
		Ok(())
	}

	/// Stops all instances of sounds, arrangements, and sequences in a group.
	pub fn stop_group(&mut self, id: GroupId, settings: StopInstanceSettings) -> AudioResult<()> {
		self.send_command_to_backend(InstanceCommand::StopGroup(id, settings))?;
		self.send_command_to_backend(SequenceCommand::StopGroup(id))?;
		Ok(())
	}

	/// Pops an event that was sent by the audio thread.
	pub fn pop_event(&mut self) -> Option<Event> {
		self.thread_channels.event_consumer.pop()
	}
}

impl Drop for AudioManager {
	fn drop(&mut self) {
		self.thread_channels
			.quit_signal_producer
			.push(true)
			.unwrap();
	}
}
