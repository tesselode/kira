mod backend;
pub(crate) mod command;
pub mod error;
pub mod renderer;
pub mod resources;

pub use backend::*;

use std::{path::Path, sync::Arc};

use ringbuf::RingBuffer;

use crate::{
	clock::{Clock, ClockHandle, ClockId},
	error::CommandError,
	parameter::{Parameter, ParameterHandle, ParameterId, Tween},
	sound::{
		data::{
			static_sound::{StaticSoundData, StaticSoundDataSettings},
			SoundData,
		},
		Sound, SoundHandle, SoundId, SoundShared,
	},
	track::{SubTrackId, Track, TrackHandle, TrackId, TrackSettings},
	value::Value,
};

use self::{
	command::{
		producer::CommandProducer, ClockCommand, Command, MixerCommand, ParameterCommand,
		SoundCommand,
	},
	error::{AddClockError, AddParameterError, AddSoundError, AddSubTrackError},
	renderer::{context::Context, Renderer, RendererState},
	resources::{create_resources, create_unused_resource_channels, ResourceControllers},
};

/// Settings for an [`AudioManager`].
pub struct AudioManagerSettings {
	/// The number of commands that be sent to the renderer at a time.
	///
	/// Each action you take, like playing a sound or pausing a parameter,
	/// queues up one command.
	pub command_capacity: usize,
	/// The maximum number of sounds that can be loaded at a time.
	pub sound_capacity: usize,
	/// The maximum number of instances of sounds that can be playing at a time.
	pub instance_capacity: usize,
	/// The maximum number of parameters that can exist at a time.
	pub parameter_capacity: usize,
	/// The maximum number of mixer sub-tracks that can exist at a time.
	pub sub_track_capacity: usize,
	/// The maximum number of clocks that can exist at a time.
	pub clock_capacity: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			sound_capacity: 100,
			command_capacity: 100,
			instance_capacity: 100,
			parameter_capacity: 100,
			sub_track_capacity: 100,
			clock_capacity: 1,
		}
	}
}

pub struct AudioManager<B: Backend> {
	backend: B,
	context: Arc<Context>,
	command_producer: CommandProducer,
	resource_controllers: ResourceControllers,
}

impl<B: Backend> AudioManager<B> {
	/// Creates a new [`AudioManager`].
	pub fn new(settings: AudioManagerSettings, mut backend: B) -> Result<Self, B::InitError> {
		let sample_rate = backend.sample_rate();
		let context = Arc::new(Context::new(sample_rate));
		let (unused_resource_producers, unused_resource_collector) =
			create_unused_resource_channels(&settings);
		let (resources, resource_controllers) =
			create_resources(&settings, unused_resource_producers, &context);
		let (command_producer, command_consumer) =
			RingBuffer::new(settings.command_capacity).split();
		let renderer = Renderer::new(context.clone(), resources, command_consumer);
		backend.init(renderer, unused_resource_collector)?;
		Ok(Self {
			backend,
			context,
			command_producer: CommandProducer::new(command_producer),
			resource_controllers,
		})
	}

	/// Returns a mutable reference to this manager's backend.
	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}

	pub fn state(&self) -> RendererState {
		self.context.state()
	}

	/// Sends a sound to the renderer and returns a handle to the sound.
	pub fn add_sound(
		&mut self,
		data: impl SoundData + 'static,
	) -> Result<SoundHandle, AddSoundError> {
		let id = SoundId(
			self.resource_controllers
				.sound_controller
				.try_reserve()
				.map_err(|_| AddSoundError::SoundLimitReached)?,
		);
		let data: Arc<dyn SoundData> = Arc::new(data);
		let shared = Arc::new(SoundShared::new());
		let sound = Sound {
			data: data.clone(),
			shared: shared.clone(),
		};
		let handle = SoundHandle {
			id,
			data,
			shared,
			instance_controller: self.resource_controllers.instance_controller.clone(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::Sound(SoundCommand::Add(id, sound)))?;
		Ok(handle)
	}

	/// Loads a sound from a file and returns a handle to the sound.
	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn load_sound(
		&mut self,
		path: impl AsRef<Path>,
		settings: StaticSoundDataSettings,
	) -> Result<SoundHandle, error::LoadSoundError> {
		let data = StaticSoundData::from_file(path, settings)?;
		let handle = self.add_sound(data)?;
		Ok(handle)
	}

	/// Creates a parameter with the specified starting value.
	pub fn add_parameter(&mut self, value: f64) -> Result<ParameterHandle, AddParameterError> {
		let id = ParameterId(
			self.resource_controllers
				.parameter_controller
				.try_reserve()
				.map_err(|_| AddParameterError::ParameterLimitReached)?,
		);
		let parameter = Parameter::new(value);
		let handle = ParameterHandle {
			id,
			shared: parameter.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::Parameter(ParameterCommand::Add(id, parameter)))?;
		Ok(handle)
	}

	/// Creates a mixer sub-track.
	pub fn add_sub_track(
		&mut self,
		settings: TrackSettings,
	) -> Result<TrackHandle, AddSubTrackError> {
		let id = SubTrackId(
			self.resource_controllers
				.sub_track_controller
				.try_reserve()
				.map_err(|_| AddSubTrackError::SubTrackLimitReached)?,
		);
		let sub_track = Track::new(settings, &self.context);
		let handle = TrackHandle {
			id: TrackId::Sub(id),
			shared: sub_track.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::Mixer(MixerCommand::AddSubTrack(id, sub_track)))?;
		Ok(handle)
	}

	/// Creates a clock.
	pub fn add_clock(&mut self, interval: impl Into<Value>) -> Result<ClockHandle, AddClockError> {
		let id = ClockId(
			self.resource_controllers
				.clock_controller
				.try_reserve()
				.map_err(|_| AddClockError::ClockLimitReached)?,
		);
		let clock = Clock::new(interval.into());
		let handle = ClockHandle {
			id,
			shared: clock.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::Clock(ClockCommand::Add(id, clock)))?;
		Ok(handle)
	}

	pub fn pause(&mut self, fade_out_tween: Tween) -> Result<(), CommandError> {
		self.command_producer.push(Command::Pause(fade_out_tween))
	}

	pub fn resume(&mut self, fade_out_tween: Tween) -> Result<(), CommandError> {
		self.command_producer.push(Command::Resume(fade_out_tween))
	}
}
