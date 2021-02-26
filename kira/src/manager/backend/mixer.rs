use basedrop::Owned;
use indexmap::IndexMap;

use crate::{
	command::MixerCommand,
	frame::Frame,
	mixer::{SendTrackId, SubTrackId, SubTrackSettings, Track, TrackIndex, TrackKind},
	parameter::Parameters,
};

pub(crate) struct Mixer {
	main_track: Track,
	// TODO: use StaticIndexMaps here
	sub_tracks: IndexMap<SubTrackId, Owned<Track>>,
	send_tracks: IndexMap<SendTrackId, Owned<Track>>,
}

impl Mixer {
	pub fn new() -> Self {
		Self {
			main_track: Track::new_normal_track(SubTrackSettings::default()),
			sub_tracks: IndexMap::new(),
			send_tracks: IndexMap::new(),
		}
	}

	pub fn run_command(&mut self, command: MixerCommand) {
		match command {
			MixerCommand::AddTrack(track) => match track.kind() {
				TrackKind::Normal { id, .. } => {
					self.sub_tracks.insert(*id, track);
				}
				TrackKind::Send { id } => {
					self.send_tracks.insert(*id, track);
				}
			},
			MixerCommand::AddEffect(index, effect, settings) => {
				match index {
					TrackIndex::Main => {
						self.main_track.add_effect(effect, settings);
					}
					TrackIndex::Sub(id) => {
						if let Some(track) = self.sub_tracks.get_mut(&id) {
							track.add_effect(effect, settings);
						}
					}
					TrackIndex::Send(id) => {
						if let Some(track) = self.send_tracks.get_mut(&id) {
							track.add_effect(effect, settings);
						}
					}
				};
			}
			MixerCommand::RemoveSubTrack(id) => {
				self.sub_tracks.remove(&id);
			}
			MixerCommand::RemoveSendTrack(id) => {
				self.send_tracks.remove(&id);
			}
			MixerCommand::SetEffectEnabled(track_index, effect_id, enabled) => {
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
			MixerCommand::RemoveEffect(track_index, effect_id) => {
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
		}
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
			if child_track.parent_track() == id.into() {
				children_input += self.process_sub_track(child_id, dt, parameters);
			}
		}
		if let Some(sub_track) = self.sub_tracks.get_mut(&id) {
			// process this track
			sub_track.add_input(children_input);
			let output = sub_track.process(dt, parameters);
			// route this track's output to send tracks
			if let TrackKind::Normal { sends, .. } = &sub_track.kind() {
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
			if track.parent_track() == TrackIndex::Main {
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
