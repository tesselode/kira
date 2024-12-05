mod builder;
mod handle;

pub use builder::*;
pub use handle::*;

use std::{error::Error, fmt::Display, sync::Arc};

use crate::{
	command::ValueChangeCommand,
	command_writers_and_readers,
	effect::Effect,
	info::Info,
	manager::backend::resources::{clocks::Clocks, modulators::Modulators, ResourceStorage},
	playback_state_manager::PlaybackStateManager,
	sound::Sound,
	tween::{Parameter, Tween},
	Decibels, Frame, StartTime,
};

use super::{SendTrack, SendTrackId, SendTrackRoute, TrackShared};

/// An error that's returned when trying to change the volume of a track route
/// that did not exist originally.
#[derive(Debug)]
pub struct NonexistentRoute;

impl Display for NonexistentRoute {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("Cannot change the volume of a track route that did not exist originally")
	}
}

impl Error for NonexistentRoute {}

pub(crate) struct Track {
	shared: Arc<TrackShared>,
	command_readers: CommandReaders,
	volume: Parameter<Decibels>,
	sounds: ResourceStorage<Box<dyn Sound>>,
	sub_tracks: ResourceStorage<Track>,
	effects: Vec<Box<dyn Effect>>,
	sends: Vec<(SendTrackId, SendTrackRoute)>,
	persist_until_sounds_finish: bool,
	playback_state_manager: PlaybackStateManager,
}

impl Track {
	fn update_shared_playback_state(&mut self) {
		self.shared
			.set_state(self.playback_state_manager.playback_state());
	}

	fn pause(&mut self, fade_out_tween: Tween) {
		self.playback_state_manager.pause(fade_out_tween);
		self.update_shared_playback_state();
	}

	fn resume(&mut self, start_time: StartTime, fade_in_tween: Tween) {
		self.playback_state_manager
			.resume(start_time, fade_in_tween);
		self.update_shared_playback_state();
	}

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
		if self
			.sub_tracks
			.iter()
			.any(|(_, sub_track)| !sub_track.should_be_removed())
		{
			return false;
		}
		if self.persist_until_sounds_finish {
			self.shared().is_marked_for_removal() && self.sounds.is_empty()
		} else {
			self.shared().is_marked_for_removal()
		}
	}

	pub fn on_start_processing(&mut self) {
		self.read_commands();
		self.sounds.remove_and_add(|sound| sound.finished());
		for (_, sound) in &mut self.sounds {
			sound.on_start_processing();
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
		clocks: &Clocks,
		modulators: &Modulators,
		send_tracks: &mut ResourceStorage<SendTrack>,
	) -> Frame {
		let info = Info::new(&clocks.0.resources, &modulators.0.resources);

		self.volume.update(dt, &info);
		for (_, route) in &mut self.sends {
			route.volume.update(dt, &info);
		}

		let changed_playback_state = self.playback_state_manager.update(dt, &info);
		if changed_playback_state {
			self.update_shared_playback_state();
		}
		if !self.playback_state_manager.playback_state().is_advancing() {
			return Frame::ZERO;
		}

		let mut output = Frame::ZERO;
		for (_, sub_track) in &mut self.sub_tracks {
			output += sub_track.process(dt, clocks, modulators, send_tracks);
		}
		for (_, sound) in &mut self.sounds {
			output += sound.process(dt, &info);
		}
		for effect in &mut self.effects {
			output = effect.process(output, dt, &info);
		}
		for (send_track_id, SendTrackRoute { volume, .. }) in &self.sends {
			let Some(send_track) = send_tracks.get_mut(send_track_id.0) else {
				continue;
			};
			send_track.add_input(output * volume.value().as_amplitude());
		}
		output *= self.volume.value().as_amplitude()
			* self.playback_state_manager.fade_volume().as_amplitude();
		output
	}

	fn read_commands(&mut self) {
		self.volume
			.read_command(&mut self.command_readers.set_volume);
		for (_, route) in &mut self.sends {
			route.read_commands();
		}
		if let Some(tween) = self.command_readers.pause.read() {
			self.pause(tween);
		}
		if let Some((start_time, tween)) = self.command_readers.resume.read() {
			self.resume(start_time, tween);
		}
	}
}

command_writers_and_readers! {
	set_volume: ValueChangeCommand<Decibels>,
	pause: Tween,
	resume: (StartTime, Tween),
}
