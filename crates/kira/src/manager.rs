//! The main entrypoint for controlling audio from gameplay code.

mod backend;
pub(crate) mod command;
pub mod error;
mod renderer;
pub(crate) mod resources;

pub use backend::*;
pub use renderer::*;
pub use resources::UnusedResourceCollector;

use std::sync::Arc;

use ringbuf::RingBuffer;

use crate::{
	clock::{Clock, ClockHandle, ClockId},
	error::CommandError,
	parameter::{Parameter, ParameterHandle, ParameterId, Tween},
	sound::SoundData,
	track::{SubTrackId, Track, TrackHandle, TrackId, TrackSettings},
	value::Value,
};

use self::{
	command::{
		producer::CommandProducer, ClockCommand, Command, MixerCommand, ParameterCommand,
		SoundCommand,
	},
	error::{AddClockError, AddParameterError, AddSubTrackError, PlaySoundError},
	renderer::context::Context,
	resources::{create_resources, create_unused_resource_channels, ResourceControllers},
};

/// Settings for an [`AudioManager`].
pub struct AudioManagerSettings {
	/// The number of commands that be sent to the renderer at a time.
	///
	/// Each action you take, like playing a sound or pausing a parameter,
	/// queues up one command.
	pub command_capacity: usize,
	/// The maximum number of sounds that can be playing at a time.
	pub sound_capacity: usize,
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
			command_capacity: 128,
			sound_capacity: 128,
			parameter_capacity: 128,
			sub_track_capacity: 128,
			clock_capacity: 1,
		}
	}
}

/// Controls audio from gameplay code.
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

	/// Returns the current playback state of the [`Renderer`].
	pub fn state(&self) -> RendererState {
		self.context.state()
	}

	/// Plays a sound.
	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		let key = self
			.resource_controllers
			.sound_controller
			.try_reserve()
			.map_err(|_| PlaySoundError::SoundLimitReached)?;
		let (sound, handle) = sound_data
			.into_sound()
			.map_err(PlaySoundError::IntoSoundError)?;
		self.command_producer
			.push(Command::Sound(SoundCommand::Add(key, sound)))?;
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

	/// Fades out and pauses the [`Renderer`].
	pub fn pause(&mut self, fade_out_tween: Tween) -> Result<(), CommandError> {
		self.command_producer.push(Command::Pause(fade_out_tween))
	}

	/// Resumes the [`Renderer`] and fades in the audio.
	pub fn resume(&mut self, fade_out_tween: Tween) -> Result<(), CommandError> {
		self.command_producer.push(Command::Resume(fade_out_tween))
	}
}
