#[cfg(test)]
mod tests;

use basedrop::Owned;

use crate::{
	command::MixerCommand,
	frame::Frame,
	mixer::{
		effect::{Effect, EffectId},
		SendTrackId, SubTrackId, Track, TrackIndex, TrackKind,
	},
	parameter::Parameters,
	static_container::index_map::StaticIndexMap,
};

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: StaticIndexMap<SubTrackId, Owned<Track>>,
	send_tracks: StaticIndexMap<SendTrackId, Owned<Track>>,
}

impl Mixer {
	pub fn new(sub_track_capacity: usize, send_track_capacity: usize) -> Self {
		Self {
			main_track: Track::new_main_track(),
			sub_tracks: StaticIndexMap::new(sub_track_capacity),
			send_tracks: StaticIndexMap::new(send_track_capacity),
		}
	}

	pub fn run_command(&mut self, command: MixerCommand) {
		match command {
			MixerCommand::AddTrack(track) => self.add_track(track),
			MixerCommand::AddEffect(index, id, effect, settings) => {
				self.add_effect(index, id, effect, settings);
			}
			MixerCommand::RemoveSubTrack(id) => {
				self.remove_sub_track(id);
			}
			MixerCommand::RemoveSendTrack(id) => {
				self.remove_send_track(id);
			}
			MixerCommand::SetEffectEnabled(track_index, effect_id, enabled) => {
				self.set_effect_enabled(track_index, effect_id, enabled);
			}
			MixerCommand::RemoveEffect(track_index, effect_id) => {
				self.remove_effect(track_index, effect_id);
			}
		}
	}

	pub fn add_track(&mut self, track: Owned<Track>) {
		match track.kind() {
			TrackKind::Main => {
				panic!("No part of the public API should be adding a main track")
			}
			TrackKind::Sub { id, .. } => {
				self.sub_tracks.try_insert(*id, track).ok();
			}
			TrackKind::Send { id } => {
				self.send_tracks.try_insert(*id, track).ok();
			}
		}
	}

	pub fn add_effect(
		&mut self,
		index: TrackIndex,
		effect_id: EffectId,
		effect: Owned<Box<dyn Effect>>,
		settings: crate::mixer::effect::EffectSettings,
	) {
		match index {
			TrackIndex::Main => {
				self.main_track.add_effect(effect_id, effect, settings);
			}
			TrackIndex::Sub(sub_track_id) => {
				if let Some(track) = self.sub_tracks.get_mut(&sub_track_id) {
					track.add_effect(effect_id, effect, settings);
				}
			}
			TrackIndex::Send(send_track_id) => {
				if let Some(track) = self.send_tracks.get_mut(&send_track_id) {
					track.add_effect(effect_id, effect, settings);
				}
			}
		};
	}

	pub fn remove_sub_track(&mut self, id: SubTrackId) {
		self.sub_tracks.remove(&id);
	}

	pub fn remove_send_track(&mut self, id: SendTrackId) {
		self.send_tracks.remove(&id);
	}

	pub fn set_effect_enabled(
		&mut self,
		track_index: TrackIndex,
		effect_id: crate::mixer::effect::EffectId,
		enabled: bool,
	) {
		match track_index {
			TrackIndex::Main => {
				if let Some(effect_slot) = self.main_track.effect_mut(effect_id) {
					effect_slot.enabled = enabled;
				}
			}
			TrackIndex::Sub(id) => {
				if let Some(track) = self.sub_tracks.get_mut(&id) {
					if let Some(effect_slot) = track.effect_mut(effect_id) {
						effect_slot.enabled = enabled;
					}
				}
			}
			TrackIndex::Send(id) => {
				if let Some(track) = self.send_tracks.get_mut(&id) {
					if let Some(effect_slot) = track.effect_mut(effect_id) {
						effect_slot.enabled = enabled;
					}
				}
			}
		};
	}

	pub fn remove_effect(
		&mut self,
		track_index: TrackIndex,
		effect_id: crate::mixer::effect::EffectId,
	) {
		match track_index {
			TrackIndex::Main => {
				self.main_track.remove_effect(effect_id);
			}
			TrackIndex::Sub(id) => {
				if let Some(track) = self.sub_tracks.get_mut(&id) {
					track.remove_effect(effect_id);
				}
			}
			TrackIndex::Send(id) => {
				if let Some(track) = self.send_tracks.get_mut(&id) {
					track.remove_effect(effect_id);
				}
			}
		};
	}

	pub fn add_input(&mut self, index: TrackIndex, input: Frame) {
		match index {
			TrackIndex::Main => {
				self.main_track.add_input(input);
			}
			TrackIndex::Sub(id) => {
				if let Some(track) = self.sub_tracks.get_mut(&id) {
					track.add_input(input);
				}
			}
			TrackIndex::Send(id) => {
				if let Some(track) = self.send_tracks.get_mut(&id) {
					track.add_input(input);
				}
			}
		}
	}

	/// Processes a sub-track.
	fn process_sub_track(&mut self, id: SubTrackId, dt: f64, parameters: &Parameters) -> Frame {
		// process all children of this sub-track and accumulate their outputs
		let mut children_input = Frame::from_mono(0.0);
		for i in 0..self.sub_tracks.len() {
			let (child_id, child_track) = self.sub_tracks.get_index(i).unwrap();
			let child_id = *child_id;
			if let Some(parent_track) = child_track.parent_track() {
				if parent_track == TrackIndex::Sub(id) {
					children_input += self.process_sub_track(child_id, dt, parameters);
				}
			}
		}
		if let Some(sub_track) = self.sub_tracks.get_mut(&id) {
			// process this track
			sub_track.add_input(children_input);
			let output = sub_track.process(dt, parameters);
			// route this track's output to send tracks
			if let TrackKind::Sub { sends, .. } = &sub_track.kind() {
				for (send_track_id, send_volume) in sends.iter() {
					if let Some(send_track) = self.send_tracks.get_mut(send_track_id) {
						send_track.add_input(output * send_volume.value() as f32);
					}
				}
				return output;
			}
		}
		Frame::from_mono(0.0)
	}

	/// Processes all top-level sub-tracks (sub-tracks that output directly
	/// to the main track) and sends their output to the main and send tracks.
	fn process_sub_tracks(&mut self, dt: f64, parameters: &Parameters) {
		for i in 0..self.sub_tracks.len() {
			let (id, track) = self.sub_tracks.get_index(i).unwrap();
			let id = *id;
			if let Some(TrackIndex::Main) = track.parent_track() {
				let output = self.process_sub_track(id, dt, parameters);
				self.main_track.add_input(output);
			}
		}
	}

	/// Processes all send tracks and sends their output to the main track.
	fn process_send_tracks(&mut self, dt: f64, parameters: &Parameters) {
		for (_, track) in &mut self.send_tracks {
			self.main_track.add_input(track.process(dt, parameters));
		}
	}

	/// Processes all tracks.
	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		self.process_sub_tracks(dt, parameters);
		self.process_send_tracks(dt, parameters);
		self.main_track.process(dt, parameters)
	}
}
