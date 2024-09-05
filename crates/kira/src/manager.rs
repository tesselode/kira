//! The main entrypoint for controlling audio from gameplay code.
//!
//! In order to play audio, you'll need to create an [`AudioManager`].
//! The [`AudioManager`] keeps track of playing sounds and manages other
//! resources like clocks, mixer tracks, and spatial scenes. Once the
//! [`AudioManager`] is dropped, its audio output will be stopped.

pub mod backend;
mod settings;

pub use backend::DefaultBackend;
pub use settings::*;

use std::sync::{
	atomic::{AtomicU32, Ordering},
	Arc,
};

use crate::{
	clock::{Clock, ClockHandle, ClockId, ClockSpeed},
	modulator::{ModulatorBuilder, ModulatorId},
	sound::SoundData,
	track::{
		MainTrackHandle, SendTrackBuilder, SendTrackHandle, SendTrackId, TrackBuilder, TrackHandle,
	},
	tween::Value,
	PlaySoundError, ResourceLimitReached,
};

use self::backend::{
	resources::{create_resources, ResourceControllers},
	Backend, Renderer, RendererShared,
};

/// Controls audio from gameplay code.
pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
	resource_controllers: ResourceControllers,
	renderer_shared: Arc<RendererShared>,
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
		track::TrackBuilder,
		effect::reverb::ReverbBuilder,
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
		let renderer_shared = Arc::new(RendererShared {
			sample_rate: AtomicU32::new(sample_rate),
		});
		let (resources, resource_controllers) = create_resources(
			settings.capacities,
			settings.main_track_builder,
			sample_rate,
		);
		let renderer = Renderer::new(renderer_shared.clone(), resources);
		backend.start(renderer)?;
		Ok(Self {
			backend,
			resource_controllers,
			renderer_shared,
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
	let sound_data = StaticSoundData::from_file("sound.ogg")?;
	manager.play(sound_data)?;
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		self.main_track().play(sound_data)
	}

	/// Creates a mixer sub-track.
	pub fn add_sub_track(
		&mut self,
		builder: TrackBuilder,
	) -> Result<TrackHandle, ResourceLimitReached> {
		let (mut track, handle) = builder.build(self.renderer_shared.clone());
		track.init_effects(self.renderer_shared.sample_rate.load(Ordering::SeqCst));
		self.resource_controllers
			.sub_track_controller
			.insert(track)?;
		Ok(handle)
	}

	/// Creates a mixer send track.
	pub fn add_send_track(
		&mut self,
		builder: SendTrackBuilder,
	) -> Result<SendTrackHandle, ResourceLimitReached> {
		let key = self
			.resource_controllers
			.send_track_controller
			.try_reserve()?;
		let id = SendTrackId(key);
		let (mut track, handle) = builder.build(id);
		track.init_effects(self.renderer_shared.sample_rate.load(Ordering::SeqCst));
		self.resource_controllers
			.send_track_controller
			.insert_with_key(key, track);
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
	) -> Result<ClockHandle, ResourceLimitReached> {
		let key = self.resource_controllers.clock_controller.try_reserve()?;
		let id = ClockId(key);
		let (clock, handle) = Clock::new(speed.into(), id);
		self.resource_controllers
			.clock_controller
			.insert_with_key(key, clock);
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
	) -> Result<Builder::Handle, ResourceLimitReached> {
		let key = self
			.resource_controllers
			.modulator_controller
			.try_reserve()?;
		let id = ModulatorId(key);
		let (modulator, handle) = builder.build(id);
		self.resource_controllers
			.modulator_controller
			.insert_with_key(key, modulator);
		Ok(handle)
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
	manager.main_track().set_volume(0.5, Tween::default());
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use]
	pub fn main_track(&mut self) -> &mut MainTrackHandle {
		&mut self.resource_controllers.main_track_handle
	}

	/// Returns the number of mixer sub-tracks that can exist at a time.
	#[must_use]
	pub fn sub_track_capacity(&self) -> u16 {
		self.resource_controllers.sub_track_controller.capacity()
	}

	/// Returns the number of mixer send tracks that can exist at a time.
	#[must_use]
	pub fn send_track_capacity(&self) -> u16 {
		self.resource_controllers.send_track_controller.capacity()
	}

	/// Returns the number of clocks that can exist at a time.
	#[must_use]
	pub fn clock_capacity(&self) -> u16 {
		self.resource_controllers.clock_controller.capacity()
	}

	/// Returns the number of modulators that can exist at a time.
	#[must_use]
	pub fn modulator_capacity(&self) -> u16 {
		self.resource_controllers.modulator_controller.capacity()
	}

	/// Returns the number of mixer sub-tracks that currently exist.
	#[must_use]
	pub fn num_sub_tracks(&self) -> u16 {
		self.resource_controllers.sub_track_controller.len()
	}

	/// Returns the number of mixer send tracks that currently exist.
	#[must_use]
	pub fn num_send_tracks(&self) -> u16 {
		self.resource_controllers.send_track_controller.len()
	}

	/// Returns the number of clocks that currently exist.
	#[must_use]
	pub fn num_clocks(&self) -> u16 {
		self.resource_controllers.clock_controller.len()
	}

	/// Returns the number of modulators that currently exist.
	#[must_use]
	pub fn num_modulators(&self) -> u16 {
		self.resource_controllers.modulator_controller.len()
	}

	/// Returns a mutable reference to this manager's backend.
	#[must_use]
	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}
}
