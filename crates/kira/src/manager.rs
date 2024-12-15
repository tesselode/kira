mod settings;

use std::sync::{atomic::Ordering, Arc};

pub use settings::*;

use crate::{
	backend::{Backend, DefaultBackend},
	clock::{Clock, ClockHandle, ClockId, ClockSpeed},
	modulator::{ModulatorBuilder, ModulatorId},
	renderer::{Renderer, RendererShared},
	resources::{
		clocks::buffered_clock::BufferedClock, create_resources,
		modulators::buffered_modulator::BufferedModulator, ResourceControllers,
	},
	sound::SoundData,
	track::{
		MainTrackHandle, SendTrackBuilder, SendTrackHandle, SendTrackId, TrackBuilder, TrackHandle,
	},
	PlaySoundError, ResourceLimitReached, Value,
};

pub struct AudioManager<B: Backend = DefaultBackend> {
	backend: B,
	resource_controllers: ResourceControllers,
	renderer_shared: Arc<RendererShared>,
}

impl<B: Backend> AudioManager<B> {
	pub fn new(settings: AudioManagerSettings<B>) -> Result<Self, B::Error> {
		let (mut backend, sample_rate) = B::setup(settings.backend_settings)?;
		let renderer_shared = Arc::new(RendererShared::new(sample_rate));
		let (resources, resource_controllers) = create_resources(
			sample_rate,
			settings.capacities,
			settings.main_track_builder,
		);
		let renderer = Renderer::new(renderer_shared.clone(), resources);
		backend.start(renderer)?;
		Ok(Self {
			backend,
			resource_controllers,
			renderer_shared,
		})
	}

	pub fn play<D: SoundData>(
		&mut self,
		sound_data: D,
	) -> Result<D::Handle, PlaySoundError<D::Error>> {
		self.resource_controllers.main_track_handle.play(sound_data)
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

	pub fn add_clock(
		&mut self,
		speed: impl Into<Value<ClockSpeed>>,
	) -> Result<ClockHandle, ResourceLimitReached> {
		let key = self.resource_controllers.clock_controller.try_reserve()?;
		let id = ClockId(key);
		let (clock, handle) = Clock::new(speed.into(), id);
		self.resource_controllers
			.clock_controller
			.insert_with_key(key, BufferedClock::new(clock));
		Ok(handle)
	}

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
			.insert_with_key(key, BufferedModulator::new(modulator));
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
	manager.main_track().set_volume(-6.0, Tween::default());
	# Result::<(), Box<dyn std::error::Error>>::Ok(())
	```
	*/
	#[must_use]
	pub fn main_track(&mut self) -> &mut MainTrackHandle {
		&mut self.resource_controllers.main_track_handle
	}

	/// Returns a mutable reference to this manager's backend.
	#[must_use]
	pub fn backend_mut(&mut self) -> &mut B {
		&mut self.backend
	}
}
