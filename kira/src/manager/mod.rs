//! Provides a bridge between the main thread and the audio thread.

mod backend;

use std::hash::Hash;

#[cfg(not(feature = "benchmarking"))]
use backend::Backend;
#[cfg(feature = "benchmarking")]
pub use backend::Backend;
use flume::{Receiver, Sender};
use indexmap::IndexSet;

use crate::{
	arrangement::{Arrangement, ArrangementHandle, ArrangementId},
	command::{
		sender::CommandSender, Command, GroupCommand, MetronomeCommand, MixerCommand,
		ParameterCommand, ResourceCommand, SequenceCommand,
	},
	error::{AudioError, AudioResult},
	group::{Group, GroupHandle, GroupId, GroupSettings},
	metronome::{Metronome, MetronomeHandle, MetronomeId, MetronomeSettings},
	mixer::{SubTrackId, Track, TrackHandle, TrackIndex, TrackSettings},
	parameter::{ParameterHandle, ParameterId, ParameterSettings},
	resource::Resource,
	sequence::{Sequence, SequenceInstanceHandle, SequenceInstanceSettings},
	sound::{Sound, SoundHandle, SoundId},
};
use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};

const RESOURCE_UNLOADER_CAPACITY: usize = 10;

/// Settings for an [`AudioManager`](crate::manager::AudioManager).
#[derive(Debug, Clone)]
#[cfg_attr(
	feature = "serde_support",
	derive(serde::Serialize, serde::Deserialize),
	serde(default)
)]
pub struct AudioManagerSettings {
	/// The number of commands that be sent to the audio thread at a time.
	///
	/// Each action you take, like starting an instance or pausing a sequence,
	/// queues up one command.
	pub num_commands: usize,
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
	/// The maximum number of groups that can be used at a time.
	pub num_groups: usize,
	/// The maximum number of audio strams that can be used at a time.
	pub num_streams: usize,
	/// The maximum number of metronomes that can be used at a time.
	pub num_metronomes: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			num_commands: 100,
			num_sounds: 100,
			num_arrangements: 100,
			num_parameters: 100,
			num_instances: 100,
			num_sequences: 25,
			num_tracks: 100,
			num_groups: 100,
			num_streams: 10,
			num_metronomes: 5,
		}
	}
}

struct ActiveIds {
	active_sound_ids: IndexSet<SoundId>,
	active_arrangement_ids: IndexSet<ArrangementId>,
	active_parameter_ids: IndexSet<ParameterId>,
	active_track_ids: IndexSet<SubTrackId>,
	active_group_ids: IndexSet<GroupId>,
	active_metronome_ids: IndexSet<MetronomeId>,
}

impl ActiveIds {
	fn new(settings: &AudioManagerSettings) -> Self {
		Self {
			active_sound_ids: IndexSet::with_capacity(settings.num_sounds),
			active_arrangement_ids: IndexSet::with_capacity(settings.num_arrangements),
			active_parameter_ids: IndexSet::with_capacity(settings.num_parameters),
			active_track_ids: IndexSet::with_capacity(settings.num_tracks),
			active_group_ids: IndexSet::with_capacity(settings.num_groups),
			active_metronome_ids: IndexSet::with_capacity(settings.num_metronomes),
		}
	}

	fn add_sound_id(&mut self, id: SoundId) -> AudioResult<()> {
		if self.active_sound_ids.len() >= self.active_sound_ids.capacity() {
			return Err(AudioError::SoundLimitReached);
		}
		self.active_sound_ids.insert(id);
		Ok(())
	}

	fn remove_sound_id(&mut self, id: SoundId) -> AudioResult<()> {
		if !self.active_sound_ids.remove(&id) {
			return Err(AudioError::NoSoundWithId(id));
		}
		Ok(())
	}

	fn add_arrangement_id(&mut self, id: ArrangementId) -> AudioResult<()> {
		if self.active_arrangement_ids.len() >= self.active_arrangement_ids.capacity() {
			return Err(AudioError::ArrangementLimitReached);
		}
		self.active_arrangement_ids.insert(id);
		Ok(())
	}

	fn remove_arrangement_id(&mut self, id: ArrangementId) -> AudioResult<()> {
		if !self.active_arrangement_ids.remove(&id) {
			return Err(AudioError::NoArrangementWithId(id));
		}
		Ok(())
	}

	fn add_parameter_id(&mut self, id: ParameterId) -> AudioResult<()> {
		if self.active_parameter_ids.len() >= self.active_parameter_ids.capacity() {
			return Err(AudioError::ParameterLimitReached);
		}
		self.active_parameter_ids.insert(id);
		Ok(())
	}

	fn remove_parameter_id(&mut self, id: ParameterId) -> AudioResult<()> {
		if !self.active_parameter_ids.remove(&id) {
			return Err(AudioError::NoParameterWithId(id));
		}
		Ok(())
	}

	fn add_track_id(&mut self, id: SubTrackId) -> AudioResult<()> {
		if self.active_track_ids.len() >= self.active_track_ids.capacity() {
			return Err(AudioError::TrackLimitReached);
		}
		self.active_track_ids.insert(id);
		Ok(())
	}

	fn remove_track_id(&mut self, id: SubTrackId) -> AudioResult<()> {
		if !self.active_track_ids.remove(&id) {
			return Err(AudioError::NoTrackWithId(id));
		}
		Ok(())
	}

	fn add_group_id(&mut self, id: GroupId) -> AudioResult<()> {
		if self.active_group_ids.len() >= self.active_group_ids.capacity() {
			return Err(AudioError::GroupLimitReached);
		}
		self.active_group_ids.insert(id);
		Ok(())
	}

	fn remove_group_id(&mut self, id: GroupId) -> AudioResult<()> {
		if !self.active_group_ids.remove(&id) {
			return Err(AudioError::NoGroupWithId(id));
		}
		Ok(())
	}

	fn add_metronome_id(&mut self, id: MetronomeId) -> AudioResult<()> {
		if self.active_metronome_ids.len() >= self.active_metronome_ids.capacity() {
			return Err(AudioError::MetronomeLimitReached);
		}
		self.active_metronome_ids.insert(id);
		Ok(())
	}

	fn remove_metronome_id(&mut self, id: MetronomeId) -> AudioResult<()> {
		if !self.active_metronome_ids.remove(&id) {
			return Err(AudioError::NoMetronomeWithId(id));
		}
		Ok(())
	}
}

/**
Plays and manages audio.

The audio manager is responsible for all communication between the gameplay thread
and the audio thread.
*/
pub struct AudioManager {
	quit_signal_sender: Sender<bool>,
	command_sender: CommandSender,
	resources_to_unload_receiver: Receiver<Resource>,
	active_ids: ActiveIds,

	// on wasm, holds the stream (as it has been created on the main thread)
	// so it can live for as long as the audio manager
	// in all cases, in benchmarking mode, we do not want an
	// audio stream anyway so we leave it out
	#[cfg(all(target_arch = "wasm32", not(feature = "benchmarking")))]
	_stream: Stream,
}

impl AudioManager {
	/// Creates a new audio manager and starts an audio thread.
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(settings: AudioManagerSettings) -> AudioResult<Self> {
		let active_ids = ActiveIds::new(&settings);
		let (quit_signal_sender, quit_signal_receiver) = flume::bounded(1);
		let (command_sender, command_receiver) = flume::bounded(settings.num_commands);
		let (unloader, resources_to_unload_receiver) = flume::bounded(RESOURCE_UNLOADER_CAPACITY);

		const WRAPPER_THREAD_SLEEP_DURATION: f64 = 1.0 / 60.0;

		let (setup_result_sender, setup_result_receiver) = flume::bounded(1);
		// set up a cpal stream on a new thread. we could do this on the main thread,
		// but that causes issues with LÖVE.
		std::thread::spawn(move || {
			match Self::setup_stream(settings, command_receiver, unloader) {
				Ok(_stream) => {
					setup_result_sender.try_send(Ok(())).unwrap();
					// wait for a quit message before ending the thread and dropping
					// the stream
					while quit_signal_receiver.try_recv().is_err() {
						std::thread::sleep(std::time::Duration::from_secs_f64(
							WRAPPER_THREAD_SLEEP_DURATION,
						));
					}
				}
				Err(error) => {
					setup_result_sender.try_send(Err(error)).unwrap();
				}
			}
		});
		// wait for the audio thread to report back a result
		loop {
			// TODO: figure out if we need to handle
			// TryRecvError::Disconnected
			if let Ok(result) = setup_result_receiver.try_recv() {
				match result {
					Ok(_) => break,
					Err(error) => return Err(error),
				}
			}
		}

		Ok(Self {
			quit_signal_sender,
			command_sender: CommandSender::new(command_sender),
			active_ids,
			resources_to_unload_receiver,
		})
	}

	/// Creates a new audio manager and starts an audio thread.
	#[cfg(target_arch = "wasm32")]
	pub fn new(settings: AudioManagerSettings) -> AudioResult<Self> {
		let active_ids = ActiveIds::new(&settings);
		let (quit_signal_sender, quit_signal_receiver) = flume::bounded(1);
		let (command_sender, command_receiver) = flume::bounded(settings.num_commands);
		let (unloader, resources_to_unload_receiver) = flume::bounded(RESOURCE_UNLOADER_CAPACITY);
		Ok(Self {
			quit_signal_sender,
			command_sender: CommandSender::new(command_sender),
			active_ids,
			resources_to_unload_receiver,
			_stream: Self::setup_stream(settings, command_receiver, unloader)?,
		})
	}

	fn setup_stream(
		settings: AudioManagerSettings,
		command_receiver: Receiver<Command>,
		unloader: Sender<Resource>,
	) -> AudioResult<Stream> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(AudioError::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate.0;
		let channels = config.channels;
		let mut backend = Backend::new(sample_rate, settings, command_receiver, unloader);
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
		let (quit_signal_sender, _) = flume::bounded(1);
		let (command_sender, command_receiver) = flume::bounded(settings.num_commands);
		let (unloader, resources_to_unload_receiver) = flume::bounded(RESOURCE_UNLOADER_CAPACITY);
		let audio_manager = Self {
			quit_signal_sender,
			command_sender: CommandSender::new(command_sender),
			active_ids: ActiveIds::new(&settings),
			resources_to_unload_receiver,
		};
		let backend = Backend::new(SAMPLE_RATE, settings, command_receiver, unloader);
		Ok((audio_manager, backend))
	}

	/// Sends a sound to the audio thread and returns a handle to the sound.
	pub fn add_sound(&mut self, sound: Sound) -> AudioResult<SoundHandle> {
		self.active_ids.add_sound_id(sound.id())?;
		let handle = SoundHandle::new(&sound, self.command_sender.clone());
		self.command_sender
			.push(ResourceCommand::AddSound(sound).into())?;
		Ok(handle)
	}

	/// Loads a sound from a file and returns a handle to the sound.
	///
	/// This is a shortcut for constructing the sound manually and adding it
	/// using [`AudioManager::add_sound`].
	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn load_sound<P: AsRef<std::path::Path>>(
		&mut self,
		path: P,
		settings: crate::sound::SoundSettings,
	) -> AudioResult<SoundHandle> {
		let sound = Sound::from_file(path, settings)?;
		self.add_sound(sound)
	}

	pub fn remove_sound(&mut self, id: impl Into<SoundId>) -> AudioResult<()> {
		let id = id.into();
		self.active_ids.remove_sound_id(id)?;
		self.command_sender
			.push(ResourceCommand::RemoveSound(id).into())
	}

	/// Sends a arrangement to the audio thread and returns a handle to the arrangement.
	pub fn add_arrangement(&mut self, arrangement: Arrangement) -> AudioResult<ArrangementHandle> {
		self.active_ids.add_arrangement_id(arrangement.id())?;
		let handle = ArrangementHandle::new(&arrangement, self.command_sender.clone());
		self.command_sender
			.push(ResourceCommand::AddArrangement(arrangement).into())?;
		Ok(handle)
	}

	pub fn remove_arrangement(&mut self, id: impl Into<ArrangementId>) -> AudioResult<()> {
		let id = id.into();
		self.active_ids.remove_arrangement_id(id)?;
		self.command_sender
			.push(ResourceCommand::RemoveArrangement(id.into()).into())
	}

	/// Frees resources that are no longer in use, such as unloaded sounds
	/// or finished sequences.
	pub fn free_unused_resources(&mut self) {
		for _ in self.resources_to_unload_receiver.try_iter() {}
	}

	pub fn add_metronome(&mut self, settings: MetronomeSettings) -> AudioResult<MetronomeHandle> {
		let id = settings.id;
		self.active_ids.add_metronome_id(id)?;
		let (event_sender, event_receiver) = flume::bounded(settings.event_queue_capacity);
		self.command_sender.push(
			MetronomeCommand::AddMetronome(id, Metronome::new(settings, event_sender)).into(),
		)?;
		Ok(MetronomeHandle::new(
			id,
			self.command_sender.clone(),
			event_receiver,
		))
	}

	pub fn remove_metronome(&mut self, id: impl Into<MetronomeId>) -> AudioResult<()> {
		let id = id.into();
		self.active_ids.remove_metronome_id(id)?;
		self.command_sender
			.push(MetronomeCommand::RemoveMetronome(id).into())
	}

	/// Starts a sequence.
	pub fn start_sequence<CustomEvent: Clone + Eq + Hash>(
		&mut self,
		sequence: Sequence<CustomEvent>,
		settings: SequenceInstanceSettings,
	) -> Result<SequenceInstanceHandle<CustomEvent>, AudioError> {
		sequence.validate()?;
		let (instance, handle) = sequence.create_instance(settings, self.command_sender.clone());
		self.command_sender
			.push(SequenceCommand::StartSequenceInstance(settings.id, instance).into())?;
		Ok(handle)
	}

	/// Creates a parameter with the specified starting value.
	pub fn add_parameter(&mut self, settings: ParameterSettings) -> AudioResult<ParameterHandle> {
		self.active_ids.add_parameter_id(settings.id)?;
		self.command_sender
			.push(ParameterCommand::AddParameter(settings.id, settings.value).into())?;
		Ok(ParameterHandle::new(
			settings.id,
			self.command_sender.clone(),
		))
	}

	pub fn remove_parameter(&mut self, id: impl Into<ParameterId>) -> AudioResult<()> {
		let id = id.into();
		self.active_ids.remove_parameter_id(id)?;
		self.command_sender
			.push(ParameterCommand::RemoveParameter(id).into())
	}

	/// Creates a mixer sub-track.
	pub fn add_sub_track(&mut self, settings: TrackSettings) -> AudioResult<TrackHandle> {
		self.active_ids.add_track_id(settings.id)?;
		let handle = TrackHandle::new(TrackIndex::Sub(settings.id), self.command_sender.clone());
		self.command_sender
			.push(MixerCommand::AddSubTrack(Track::new(settings)).into())?;
		Ok(handle)
	}

	/// Removes a sub-track from the mixer.
	pub fn remove_sub_track(&mut self, id: SubTrackId) -> AudioResult<()> {
		let id = id.into();
		self.active_ids.remove_track_id(id)?;
		self.command_sender
			.push(MixerCommand::RemoveSubTrack(id).into())
	}

	/// Adds a group.
	pub fn add_group(&mut self, settings: GroupSettings) -> AudioResult<GroupHandle> {
		let id = settings.id;
		self.active_ids.add_group_id(id)?;
		self.command_sender
			.push(GroupCommand::AddGroup(id, Group::new(settings)).into())?;
		Ok(GroupHandle::new(id, self.command_sender.clone()))
	}

	/// Removes a group.
	pub fn remove_group(&mut self, id: impl Into<GroupId>) -> AudioResult<()> {
		let id = id.into();
		self.active_ids.remove_group_id(id)?;
		self.command_sender
			.push(GroupCommand::RemoveGroup(id).into())
	}
}

impl Drop for AudioManager {
	fn drop(&mut self) {
		// TODO: add proper error handling here without breaking benchmarks
		self.quit_signal_sender.send(true).ok();
	}
}
