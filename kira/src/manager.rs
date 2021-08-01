pub mod backend;
pub(crate) mod command;
pub mod renderer;
pub mod resources;

use std::{path::Path, sync::Arc};

use ringbuf::RingBuffer;

use crate::{
	error::{AddParameterError, AddSoundError, AddSubTrackError},
	parameter::{handle::ParameterHandle, Parameter, ParameterId},
	sound::{
		data::{
			static_sound::{settings::StaticSoundDataSettings, StaticSoundData},
			SoundData,
		},
		handle::SoundHandle,
		Sound, SoundId, SoundShared,
	},
	track::{handle::TrackHandle, settings::TrackSettings, SubTrackId, Track, TrackId},
};

use self::{
	backend::Backend,
	command::{producer::CommandProducer, Command, MixerCommand, ParameterCommand, SoundCommand},
	renderer::{context::Context, Renderer},
	resources::{create_resources, create_unused_resource_channels, ResourceControllers},
};

pub struct AudioManagerSettings {
	pub sound_capacity: usize,
	pub command_capacity: usize,
	pub instance_capacity: usize,
	pub parameter_capacity: usize,
	pub sub_track_capacity: usize,
}

impl Default for AudioManagerSettings {
	fn default() -> Self {
		Self {
			sound_capacity: 100,
			command_capacity: 100,
			instance_capacity: 100,
			parameter_capacity: 100,
			sub_track_capacity: 100,
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

	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}

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

	#[cfg(any(feature = "mp3", feature = "ogg", feature = "flac", feature = "wav"))]
	pub fn load_sound(
		&mut self,
		path: impl AsRef<Path>,
		settings: StaticSoundDataSettings,
	) -> Result<SoundHandle, crate::error::LoadSoundError> {
		let data = StaticSoundData::from_file(path, settings)?;
		let handle = self.add_sound(data)?;
		Ok(handle)
	}

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
}
