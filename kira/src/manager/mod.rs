//! Provides a bridge between the main thread and the audio thread.

mod backend;

use std::{hash::Hash, path::Path};

#[cfg(not(feature = "benchmarking"))]
use backend::Backend;
#[cfg(feature = "benchmarking")]
pub use backend::Backend;

use crate::{
	arrangement::{Arrangement, ArrangementHandle, ArrangementId},
	audio_stream::{AudioStream, AudioStreamId},
	command::{
		sender::CommandSender, GroupCommand, InstanceCommand, MetronomeCommand, MixerCommand,
		ParameterCommand, ResourceCommand, SequenceCommand, StreamCommand,
	},
	error::{AudioError, AudioResult},
	group::{Group, GroupId},
	instance::{PauseInstanceSettings, ResumeInstanceSettings, StopInstanceSettings},
	metronome::MetronomeSettings,
	mixer::{
		effect::{Effect, EffectId, EffectSettings},
		effect_slot::EffectSlot,
		SubTrackId, Track, TrackIndex, TrackSettings,
	},
	parameter::{ParameterHandle, ParameterId, Tween},
	playable::PlayableSettings,
	sequence::SequenceInstance,
	sequence::{Sequence, SequenceInstanceHandle, SequenceInstanceId, SequenceInstanceSettings},
	sound::{Sound, SoundHandle, SoundId},
	tempo::Tempo,
	util::index_set_from_vec,
	value::Value,
	Event,
};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Receiver, RingBuffer, Sender};

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
	pub quit_signal_sender: Sender<bool>,
	pub command_sender: CommandSender,
	pub event_receiver: Receiver<Event>,
	pub sounds_to_unload_receiver: Receiver<Sound>,
	pub arrangements_to_unload_receiver: Receiver<Arrangement>,
	pub sequence_instances_to_unload_receiver: Receiver<SequenceInstance>,
	pub tracks_to_unload_receiver: Receiver<Track>,
	pub effect_slots_to_unload_receiver: Receiver<EffectSlot>,
	pub groups_to_unload_receiver: Receiver<Group>,
	pub streams_to_unload_receiver: Receiver<Box<dyn AudioStream>>,
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
		let (audio_manager_thread_channels, backend_thread_channels, mut quit_signal_receiver) =
			Self::create_thread_channels(&settings);

		let stream = {
			let (mut setup_result_sender, mut setup_result_receiver) =
				RingBuffer::<AudioResult<()>>::new(1).split();
			// set up a cpal stream on a new thread. we could do this on the main thread,
			// but that causes issues with LÖVE.
			std::thread::spawn(move || {
				match Self::setup_stream(settings, backend_thread_channels) {
					Ok(_stream) => {
						setup_result_sender.push(Ok(())).unwrap();
						// wait for a quit message before ending the thread and dropping
						// the stream
						while let None = quit_signal_receiver.pop() {
							std::thread::sleep(std::time::Duration::from_secs_f64(
								WRAPPER_THREAD_SLEEP_DURATION,
							));
						}
					}
					Err(error) => {
						setup_result_sender.push(Err(error)).unwrap();
					}
				}
			});
			// wait for the audio thread to report back a result
			loop {
				if let Some(result) = setup_result_receiver.pop() {
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
		Receiver<bool>,
	) {
		let (quit_signal_sender, quit_signal_receiver) = RingBuffer::new(1).split();
		let (command_sender, command_receiver) = RingBuffer::new(settings.num_commands).split();
		let (sounds_to_unload_sender, sounds_to_unload_receiver) =
			RingBuffer::new(settings.num_sounds).split();
		let (arrangements_to_unload_sender, arrangements_to_unload_receiver) =
			RingBuffer::new(settings.num_arrangements).split();
		let (sequence_instances_to_unload_sender, sequence_instances_to_unload_receiver) =
			RingBuffer::new(settings.num_sequences).split();
		let (tracks_to_unload_sender, tracks_to_unload_receiver) =
			RingBuffer::new(settings.num_tracks).split();
		let (effect_slots_to_unload_sender, effect_slots_to_unload_receiver) =
			RingBuffer::new(settings.num_tracks * settings.num_effects_per_track).split();
		let (groups_to_unload_sender, groups_to_unload_receiver) =
			RingBuffer::new(settings.num_groups).split();
		let (event_sender, event_receiver) = RingBuffer::new(settings.num_events).split();
		let (streams_to_unload_sender, streams_to_unload_receiver) =
			RingBuffer::new(settings.num_streams).split();
		let audio_manager_thread_channels = AudioManagerThreadChannels {
			quit_signal_sender,
			command_sender: CommandSender::new(command_sender),
			event_receiver,
			sounds_to_unload_receiver,
			arrangements_to_unload_receiver,
			sequence_instances_to_unload_receiver,
			tracks_to_unload_receiver,
			effect_slots_to_unload_receiver,
			groups_to_unload_receiver,
			streams_to_unload_receiver,
		};
		let backend_thread_channels = BackendThreadChannels {
			command_receiver,
			event_sender,
			sounds_to_unload_sender,
			arrangements_to_unload_sender,
			sequence_instances_to_unload_sender,
			tracks_to_unload_sender,
			effect_slots_to_unload_sender,
			groups_to_unload_sender,
			streams_to_unload_sender,
		};
		(
			audio_manager_thread_channels,
			backend_thread_channels,
			quit_signal_receiver,
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
	/// Creates an [`AudioManager`] and [`Backend`] without sending
	/// the backend to another thread.
	///
	/// This is useful for updating the backend manually for
	/// benchmarking.
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

	/// Sends a sound to the audio thread and returns a handle to the sound.
	pub fn add_sound(&mut self, sound: Sound) -> AudioResult<SoundHandle> {
		let id = SoundId::new(&sound);
		self.thread_channels
			.command_sender
			.push(ResourceCommand::AddSound(id, sound).into())?;
		Ok(SoundHandle::new(
			id,
			self.thread_channels.command_sender.clone(),
		))
	}

	/// Loads a sound from a file and returns a handle to the sound.
	///
	/// This is a shortcut for constructing the sound manually and adding it
	/// using [`AudioManager::add_sound`].
	pub fn load_sound<P: AsRef<Path>>(
		&mut self,
		path: P,
		settings: PlayableSettings,
	) -> AudioResult<SoundHandle> {
		let sound = Sound::from_file(path, settings)?;
		self.add_sound(sound)
	}

	/// Sends a arrangement to the audio thread and returns a handle to the arrangement.
	pub fn add_arrangement(&mut self, arrangement: Arrangement) -> AudioResult<ArrangementHandle> {
		let id = ArrangementId::new(&arrangement);
		self.thread_channels
			.command_sender
			.push(ResourceCommand::AddArrangement(id, arrangement).into())?;
		Ok(ArrangementHandle::new(
			id,
			self.thread_channels.command_sender.clone(),
		))
	}

	/// Frees resources that are no longer in use, such as unloaded sounds
	/// or finished sequences.
	pub fn free_unused_resources(&mut self) {
		while let Some(_) = self.thread_channels.sounds_to_unload_receiver.pop() {}
		while let Some(_) = self.thread_channels.arrangements_to_unload_receiver.pop() {}
		while let Some(_) = self
			.thread_channels
			.sequence_instances_to_unload_receiver
			.pop()
		{}
		while let Some(_) = self.thread_channels.tracks_to_unload_receiver.pop() {}
		while let Some(_) = self.thread_channels.effect_slots_to_unload_receiver.pop() {}
		while let Some(_) = self.thread_channels.groups_to_unload_receiver.pop() {}
		while let Some(_) = self.thread_channels.streams_to_unload_receiver.pop() {}
	}

	/// Sets the tempo of the metronome.
	pub fn set_metronome_tempo<T: Into<Value<Tempo>>>(
		&mut self,
		tempo: T,
	) -> Result<(), AudioError> {
		self.thread_channels
			.command_sender
			.push(MetronomeCommand::SetMetronomeTempo(tempo.into()).into())
	}

	/// Starts or resumes the metronome.
	pub fn start_metronome(&mut self) -> Result<(), AudioError> {
		self.thread_channels
			.command_sender
			.push(MetronomeCommand::StartMetronome.into())
	}

	/// Pauses the metronome.
	pub fn pause_metronome(&mut self) -> Result<(), AudioError> {
		self.thread_channels
			.command_sender
			.push(MetronomeCommand::PauseMetronome.into())
	}

	/// Stops and resets the metronome.
	pub fn stop_metronome(&mut self) -> Result<(), AudioError> {
		self.thread_channels
			.command_sender
			.push(MetronomeCommand::StopMetronome.into())
	}

	/// Starts a sequence.
	pub fn start_sequence<CustomEvent: Clone + Eq + Hash>(
		&mut self,
		sequence: Sequence<CustomEvent>,
		settings: SequenceInstanceSettings,
	) -> Result<SequenceInstanceHandle<CustomEvent>, AudioError> {
		sequence.validate()?;
		let id = SequenceInstanceId::new();
		let (instance, handle) =
			sequence.create_instance(settings, id, self.thread_channels.command_sender.clone());
		self.thread_channels
			.command_sender
			.push(SequenceCommand::StartSequenceInstance(id, instance).into())?;
		Ok(handle)
	}

	/// Creates a parameter with the specified starting value.
	pub fn add_parameter(&mut self, value: f64) -> AudioResult<ParameterHandle> {
		let id = ParameterId::new();
		self.thread_channels
			.command_sender
			.push(ParameterCommand::AddParameter(id, value).into())?;
		Ok(ParameterHandle::new(
			id,
			self.thread_channels.command_sender.clone(),
		))
	}

	/// Creates a mixer sub-track.
	pub fn add_sub_track(&mut self, settings: TrackSettings) -> AudioResult<SubTrackId> {
		let id = SubTrackId::new();
		self.thread_channels
			.command_sender
			.push(MixerCommand::AddSubTrack(id, Track::new(settings)).into())?;
		Ok(id)
	}

	/// Removes a sub-track from the mixer.
	pub fn remove_sub_track(&mut self, id: SubTrackId) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(MixerCommand::RemoveSubTrack(id).into())
	}

	/// Adds an effect to a track.
	pub fn add_effect_to_track<T: Into<TrackIndex> + Copy, E: Effect + 'static>(
		&mut self,
		track_index: T,
		effect: E,
		settings: EffectSettings,
	) -> AudioResult<EffectId> {
		let effect_id = EffectId::new(track_index.into());
		self.thread_channels.command_sender.push(
			MixerCommand::AddEffect(track_index.into(), effect_id, Box::new(effect), settings)
				.into(),
		)?;
		Ok(effect_id)
	}

	/// Removes an effect from the mixer.
	pub fn remove_effect(&mut self, effect_id: EffectId) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(MixerCommand::RemoveEffect(effect_id).into())
	}

	/// Starts an audio stream on the specified track.
	pub fn add_stream<T: Into<TrackIndex>, S: AudioStream>(
		&mut self,
		track_index: T,
		stream: S,
	) -> AudioResult<AudioStreamId> {
		let stream_id = AudioStreamId::new();
		self.thread_channels
			.command_sender
			.push(StreamCommand::AddStream(stream_id, track_index.into(), Box::new(stream)).into())
			.map(|()| stream_id)
	}

	/// Stops and drops the specified audio stream.
	pub fn remove_stream(&mut self, stream_id: AudioStreamId) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(StreamCommand::RemoveStream(stream_id).into())
	}

	/// Adds a group.
	pub fn add_group<T: Into<Vec<GroupId>>>(&mut self, parent_groups: T) -> AudioResult<GroupId> {
		let id = GroupId::new();
		let group = Group::new(index_set_from_vec(parent_groups.into()));
		self.thread_channels
			.command_sender
			.push(GroupCommand::AddGroup(id, group).into())?;
		Ok(id)
	}

	/// Removes a group.
	pub fn remove_group(&mut self, id: GroupId) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(GroupCommand::RemoveGroup(id).into())
	}

	/// Pauses all instances of sounds, arrangements, and sequences in a group.
	pub fn pause_group(&mut self, id: GroupId, settings: PauseInstanceSettings) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(InstanceCommand::PauseGroup(id, settings).into())?;
		self.thread_channels
			.command_sender
			.push(SequenceCommand::PauseGroup(id).into())?;
		Ok(())
	}

	/// Resumes all instances of sounds, arrangements, and sequences in a group.
	pub fn resume_group(
		&mut self,
		id: GroupId,
		settings: ResumeInstanceSettings,
	) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(InstanceCommand::ResumeGroup(id, settings).into())?;
		self.thread_channels
			.command_sender
			.push(SequenceCommand::ResumeGroup(id).into())?;
		Ok(())
	}

	/// Stops all instances of sounds, arrangements, and sequences in a group.
	pub fn stop_group(&mut self, id: GroupId, settings: StopInstanceSettings) -> AudioResult<()> {
		self.thread_channels
			.command_sender
			.push(InstanceCommand::StopGroup(id, settings).into())?;
		self.thread_channels
			.command_sender
			.push(SequenceCommand::StopGroup(id).into())?;
		Ok(())
	}

	/// Pops an event that was sent by the audio thread.
	pub fn pop_event(&mut self) -> Option<Event> {
		self.thread_channels.event_receiver.pop()
	}
}

impl Drop for AudioManager {
	fn drop(&mut self) {
		self.thread_channels.quit_signal_sender.push(true).unwrap();
	}
}
