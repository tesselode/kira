mod builder;
mod error;
mod handle;
mod main_track_builder;
mod main_track_handle;
mod send_track_builder;
mod send_track_handle;

pub use builder::*;
pub use error::*;
pub use handle::*;
pub use main_track_builder::*;
pub use main_track_handle::*;
pub use send_track_builder::*;
pub use send_track_handle::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use crate::{
	arena::Key,
	effect::Effect,
	manager::backend::resources::{spatial_scenes::SpatialScenes, ResourceStorage},
	sound::Sound,
	spatial::listener::Listener,
};

use crate::{
	clock::clock_info::ClockInfoProvider,
	command::{CommandReader, ValueChangeCommand},
	frame::Frame,
	modulator::value_provider::ModulatorValueProvider,
	tween::Parameter,
	Volume,
};

/// A unique identifier for a mixer send track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SendTrackId(pub(crate) Key);

impl From<&SendTrackHandle> for SendTrackId {
	fn from(handle: &SendTrackHandle) -> Self {
		handle.id()
	}
}

pub(crate) struct MainTrack {
	volume: Parameter<Volume>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Volume>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	listeners: ResourceStorage<Listener>,
	effects: Vec<Box<dyn Effect>>,
}

impl MainTrack {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		self.listeners
			.remove_and_add(|listener| listener.finished());
		for (_, listener) in &mut self.listeners {
			listener.on_start_processing();
		}
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		input: Frame,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		spatial_scenes: &SpatialScenes,
	) -> Frame {
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		let mut output = input;
		for (_, sound) in &mut self.sounds {
			output += sound.process(dt, clock_info_provider, modulator_value_provider);
		}
		for (_, listener) in &mut self.listeners {
			output += listener.process(
				dt,
				clock_info_provider,
				modulator_value_provider,
				spatial_scenes,
			);
		}
		for effect in &mut self.effects {
			output = effect.process(output, dt, clock_info_provider, modulator_value_provider);
		}
		output *= self.volume.value().as_amplitude() as f32;
		output
	}
}

pub(crate) struct Track {
	shared: Arc<TrackShared>,
	volume: Parameter<Volume>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Volume>>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	listeners: ResourceStorage<Listener>,
	sub_tracks: ResourceStorage<Track>,
	effects: Vec<Box<dyn Effect>>,
	sends: Vec<(SendTrackId, SendTrackRoute)>,
}

impl Track {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate);
		}
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.init_effects(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.on_change_sample_rate(sample_rate);
		}
	}

	#[must_use]
	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn should_be_removed(&self) -> bool {
		self.shared().is_marked_for_removal()
			&& self
				.sub_tracks
				.iter()
				.all(|(_, sub_track)| sub_track.should_be_removed())
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		for (_, route) in &mut self.sends {
			route.read_commands();
		}
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
		}
		self.listeners
			.remove_and_add(|listener| listener.finished());
		for (_, listener) in &mut self.listeners {
			listener.on_start_processing();
		}
		self.sub_tracks
			.remove_and_add(|sub_track| sub_track.should_be_removed());
		for (_, sub_track) in &mut self.sub_tracks {
			sub_track.on_start_processing();
		}
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		spatial_scenes: &SpatialScenes,
		send_tracks: &mut ResourceStorage<SendTrack>,
	) -> Frame {
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		for (_, route) in &mut self.sends {
			route
				.volume
				.update(dt, clock_info_provider, modulator_value_provider);
		}

		let mut output = Frame::ZERO;
		for (_, sub_track) in &mut self.sub_tracks {
			output += sub_track.process(
				dt,
				clock_info_provider,
				modulator_value_provider,
				spatial_scenes,
				send_tracks,
			);
		}
		for (_, sound) in &mut self.sounds {
			output += sound.process(dt, clock_info_provider, modulator_value_provider);
		}
		for (_, listener) in &mut self.listeners {
			output += listener.process(
				dt,
				clock_info_provider,
				modulator_value_provider,
				spatial_scenes,
			);
		}
		for effect in &mut self.effects {
			output = effect.process(output, dt, clock_info_provider, modulator_value_provider);
		}
		output *= self.volume.value().as_amplitude() as f32;
		for (send_track_id, SendTrackRoute { volume, .. }) in &self.sends {
			let Some(send_track) = send_tracks.get_mut(send_track_id.0) else {
				continue;
			};
			send_track.add_input(output * volume.value().as_amplitude() as f32);
		}
		output
	}
}

pub(crate) struct SendTrack {
	shared: Arc<TrackShared>,
	volume: Parameter<Volume>,
	set_volume_command_reader: CommandReader<ValueChangeCommand<Volume>>,
	effects: Vec<Box<dyn Effect>>,
	input: Frame,
}

impl SendTrack {
	pub fn init_effects(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.init(sample_rate);
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		for effect in &mut self.effects {
			effect.on_change_sample_rate(sample_rate);
		}
	}

	#[must_use]
	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn on_start_processing(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
		for effect in &mut self.effects {
			effect.on_start_processing();
		}
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.volume
			.update(dt, clock_info_provider, modulator_value_provider);
		let mut output = std::mem::replace(&mut self.input, Frame::ZERO);
		for effect in &mut self.effects {
			output = effect.process(output, dt, clock_info_provider, modulator_value_provider);
		}
		output * self.volume.value().as_amplitude() as f32
	}
}

pub(crate) struct SendTrackRoute {
	pub(crate) volume: Parameter<Volume>,
	pub(crate) set_volume_command_reader: CommandReader<ValueChangeCommand<Volume>>,
}

impl SendTrackRoute {
	pub fn read_commands(&mut self) {
		self.volume
			.read_command(&mut self.set_volume_command_reader);
	}
}

#[derive(Debug)]
pub(crate) struct TrackShared {
	removed: AtomicBool,
}

impl TrackShared {
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	#[must_use]
	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}
