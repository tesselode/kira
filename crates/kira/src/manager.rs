//! The main entrypoint for controlling audio from gameplay code.
//!
//! In order to play audio, you'll need to create an [`AudioManager`].
//! The [`AudioManager`] keeps track of playing sounds and manages other
//! resources like clocks, mixer tracks, and spatial scenes. Once the
//! [`AudioManager`] is dropped, its audio output will be stopped.

pub mod backend;
pub(crate) mod command;
pub mod error;
mod settings;

use ringbuf::HeapRb;
pub use settings::*;

use std::{collections::HashSet, sync::Arc};

use crate::{
	clock::{Clock, ClockHandle, ClockId, ClockSpeed},
	error::CommandError,
	manager::command::ModulatorCommand,
	modulator::{ModulatorBuilder, ModulatorId},
	sound::SoundData,
	spatial::scene::{SpatialScene, SpatialSceneHandle, SpatialSceneId, SpatialSceneSettings},
	track::{SubTrackId, Track, TrackBuilder, TrackHandle, TrackId},
	tween::{Tween, Value},
};

use self::{
	backend::{
		resources::{
			create_resources, create_unused_resource_channels, ResourceControllers,
			UnusedResourceConsumers,
		},
		Backend, DefaultBackend, Renderer, RendererShared,
	},
	command::{
		producer::CommandProducer, ClockCommand, Command, MixerCommand, SoundCommand,
		SpatialSceneCommand,
	},
	error::{
		AddClockError, AddModulatorError, AddSpatialSceneError, AddSubTrackError, PlaySoundError,
	},
};

/// The playback state for all audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MainPlaybackState {
	/// Audio is playing normally.
	Playing,
	/// Audio is fading out and will be paused when the
	/// fade-out is finished.
	Pausing,
	/// Audio processing is paused and no sound is being
	/// produced.
	Paused,
}

impl MainPlaybackState {
	fn from_u8(state: u8) -> Self {
		match state {
			0 => Self::Playing,
			1 => Self::Pausing,
			2 => Self::Paused,
			_ => panic!("Not a valid MainPlaybackState"),
		}
	}
}

/// Controls audio from gameplay code.
pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
	renderer_shared: Arc<RendererShared>,
	command_producer: CommandProducer,
	resource_controllers: ResourceControllers,
	unused_resource_consumers: UnusedResourceConsumers,
}

impl<B: Backend> AudioManager<B> {
	/**
	Creates a new [`AudioManager`].

	# Examples

	Create an [`AudioManager`] using the [`DefaultBackend`] and the
	default settings:

	```no_run
	use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};

	let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Create an [`AudioManager`] with a reverb effect on the main mixer track:

	```no_run
	use kira::{
		manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend},
		track::{TrackBuilder, effect::reverb::ReverbBuilder},
	};

	let audio_manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings {
		main_track_builder: TrackBuilder::new().with_effect(ReverbBuilder::new()),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn new(settings: AudioManagerSettings<B>) -> Result<Self, B::Error> {
		let (mut backend, sample_rate) = B::setup(settings.backend_settings)?;
		let (command_producer, command_consumer) =
			HeapRb::new(settings.capacities.command_capacity).split();
		let (unused_resource_producers, unused_resource_consumers) =
			create_unused_resource_channels(settings.capacities);
		let (resources, resource_controllers) = create_resources(
			settings.capacities,
			settings.main_track_builder,
			unused_resource_producers,
			sample_rate,
		);
		let renderer = Renderer::new(sample_rate, resources, command_consumer);
		let renderer_shared = renderer.shared();
		backend.start(renderer)?;
		Ok(Self {
			backend,
			renderer_shared,
			command_producer: CommandProducer::new(command_producer),
			resource_controllers,
			unused_resource_consumers,
		})
	}

	/**
	Plays a sound.

	# Examples

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# };
	use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let sound_data = StaticSoundData::from_file("sound.ogg", StaticSoundSettings::default())?;
	manager.play(sound_data)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		while self
			.unused_resource_consumers
			.sound
			.lock()
			.expect("unused resource consumer mutex poisoned")
			.pop()
			.is_some()
		{}
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

	/// Creates a mixer sub-track.
	pub fn add_sub_track(
		&mut self,
		builder: TrackBuilder,
	) -> Result<TrackHandle, AddSubTrackError> {
		let unused_sub_track_consumer = &mut self
			.unused_resource_consumers
			.sub_track
			.lock()
			.expect("unused resource consumer mutex poisoned");
		while unused_sub_track_consumer.pop().is_some() {}
		let id = SubTrackId(
			self.resource_controllers
				.sub_track_controller
				.try_reserve()
				.map_err(|_| AddSubTrackError::SubTrackLimitReached)?,
		);
		let existing_routes = builder.routes.0.keys().copied().collect();
		let sub_track = Track::new(builder);
		let handle = TrackHandle {
			id: TrackId::Sub(id),
			shared: Some(sub_track.shared()),
			command_producer: self.command_producer.clone(),
			existing_routes,
		};
		self.command_producer
			.push(Command::Mixer(MixerCommand::AddSubTrack(id, sub_track)))?;
		Ok(handle)
	}

	/**
	Creates a clock.

	# Examples

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# 	clock::ClockSpeed
	# };

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let clock = manager.add_clock(ClockSpeed::TicksPerMinute(120.0))?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn add_clock(
		&mut self,
		speed: impl Into<Value<ClockSpeed>>,
	) -> Result<ClockHandle, AddClockError> {
		let unused_clock_consumer = &mut self
			.unused_resource_consumers
			.clock
			.lock()
			.expect("unused resource consumer mutex poisoned");
		while unused_clock_consumer.pop().is_some() {}
		let id = ClockId(
			self.resource_controllers
				.clock_controller
				.try_reserve()
				.map_err(|_| AddClockError::ClockLimitReached)?,
		);
		let clock = Clock::new(speed.into());
		let handle = ClockHandle {
			id,
			shared: clock.shared(),
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::Clock(ClockCommand::Add(id, clock)))?;
		Ok(handle)
	}

	/// Creates a spatial scene.
	pub fn add_spatial_scene(
		&mut self,
		settings: SpatialSceneSettings,
	) -> Result<SpatialSceneHandle, AddSpatialSceneError> {
		let unused_spatial_scene_consumer = &mut self
			.unused_resource_consumers
			.spatial_scene
			.lock()
			.expect("unused resource consumer mutex poisoned");
		while unused_spatial_scene_consumer.pop().is_some() {}
		let id = SpatialSceneId(
			self.resource_controllers
				.spatial_scene_controller
				.try_reserve()
				.map_err(|_| AddSpatialSceneError::SpatialSceneLimitReached)?,
		);
		let (spatial_scene, unused_emitter_consumer, unused_listener_consumer) =
			SpatialScene::new(settings);
		let handle = SpatialSceneHandle {
			id,
			shared: spatial_scene.shared(),
			emitter_controller: spatial_scene.emitter_controller(),
			unused_emitter_consumer,
			listener_controller: spatial_scene.listener_controller(),
			unused_listener_consumer,
			command_producer: self.command_producer.clone(),
		};
		self.command_producer
			.push(Command::SpatialScene(SpatialSceneCommand::Add(
				id,
				spatial_scene,
			)))?;
		Ok(handle)
	}

	/**
	Creates a modulator.

	# Examples

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# };
	use kira::modulator::lfo::LfoBuilder;

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	let modulator = manager.add_modulator(LfoBuilder::new())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn add_modulator<Builder: ModulatorBuilder>(
		&mut self,
		builder: Builder,
	) -> Result<Builder::Handle, AddModulatorError> {
		let unused_modulator_consumer = &mut self
			.unused_resource_consumers
			.modulator
			.lock()
			.expect("unused resource consumer mutex poisoned");
		while unused_modulator_consumer.pop().is_some() {}
		let id = ModulatorId(
			self.resource_controllers
				.modulator_controller
				.try_reserve()
				.map_err(|_| AddModulatorError::ModulatorLimitReached)?,
		);
		let (modulator, handle) = builder.build(id);
		self.command_producer
			.push(Command::Modulator(ModulatorCommand::Add(id, modulator)))?;
		Ok(handle)
	}

	/**
	Fades out and pauses all audio.

	# Examples

	Pause audio immediately:

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# };
	use kira::tween::Tween;

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	manager.pause(Tween::default())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```

	Fade out audio for 3 seconds and then pause:

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# };
	use kira::tween::Tween;
	use std::time::Duration;

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	manager.pause(Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn pause(&self, fade_out_tween: Tween) -> Result<(), CommandError> {
		self.command_producer.push(Command::Pause(fade_out_tween))
	}

	/**
	Resumes and fades in all audio.

	# Examples

	Resume audio with a 3-second fade-in:

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# };
	use kira::tween::Tween;
	use std::time::Duration;

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	manager.resume(Tween {
		duration: Duration::from_secs(3),
		..Default::default()
	})?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn resume(&self, fade_out_tween: Tween) -> Result<(), CommandError> {
		self.command_producer.push(Command::Resume(fade_out_tween))
	}

	/**
	Returns a handle to the main mixer track.

	# Examples

	Use the main track handle to adjust the volume of all audio:

	```no_run
	# use kira::{
	# 	manager::{
	# 		AudioManager, AudioManagerSettings,
	# 		backend::DefaultBackend,
	# 	},
	# };
	use kira::tween::Tween;

	# let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
	manager.main_track().set_volume(0.5, Tween::default())?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn main_track(&self) -> TrackHandle {
		TrackHandle {
			id: TrackId::Main,
			shared: None,
			command_producer: self.command_producer.clone(),
			existing_routes: HashSet::new(),
		}
	}

	/// Returns the current playback state of the audio.
	pub fn state(&self) -> MainPlaybackState {
		self.renderer_shared.state()
	}

	/// Returns the number of sounds that can be loaded at a time.
	pub fn sound_capacity(&self) -> usize {
		self.resource_controllers.sound_controller.capacity()
	}

	/// Returns the number of mixer sub-tracks that can exist at a time.
	pub fn sub_track_capacity(&self) -> usize {
		self.resource_controllers.sub_track_controller.capacity()
	}

	/// Returns the number of clocks that can exist at a time.
	pub fn clock_capacity(&self) -> usize {
		self.resource_controllers.clock_controller.capacity()
	}

	/// Returns the number of spatial scenes that can exist at a time.
	pub fn spatial_scene_capacity(&self) -> usize {
		self.resource_controllers
			.spatial_scene_controller
			.capacity()
	}

	/// Returns the number of modulators that can exist at a time.
	pub fn modulator_capacity(&self) -> usize {
		self.resource_controllers.modulator_controller.capacity()
	}

	/// Returns the number of sounds that are currently loaded.
	pub fn num_sounds(&self) -> usize {
		self.resource_controllers.sound_controller.len()
	}

	/// Returns the number of mixer sub-tracks that currently exist.
	pub fn num_sub_tracks(&self) -> usize {
		self.resource_controllers.sub_track_controller.len()
	}

	/// Returns the number of clocks that currently exist.
	pub fn num_clocks(&self) -> usize {
		self.resource_controllers.clock_controller.len()
	}

	/// Returns the number of spatial scenes that currently exist.
	pub fn num_spatial_scenes(&self) -> usize {
		self.resource_controllers.spatial_scene_controller.len()
	}

	/// Returns the number of modulators that currently exist.
	pub fn num_modulators(&self) -> usize {
		self.resource_controllers.modulator_controller.len()
	}

	/// Returns a mutable reference to this manager's backend.
	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}
}
