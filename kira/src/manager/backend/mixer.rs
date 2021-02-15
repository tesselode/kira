use basedrop::Owned;
use indexmap::IndexMap;

use crate::{
	command::MixerCommand,
	frame::Frame,
	mixer::{SubTrackId, Track, TrackIndex, TrackSettings},
	parameter::Parameters,
};

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: IndexMap<SubTrackId, Owned<Track>>,
}

impl Mixer {
	pub fn new() -> Self {
		Self {
			main_track: Track::new(TrackSettings::default()),
			sub_tracks: IndexMap::new(),
		}
	}

	pub fn run_command(&mut self, command: MixerCommand) {
		match command {
			MixerCommand::AddSubTrack(track) => {
				self.sub_tracks.insert(track.id(), track);
			}
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
				};
			}
			MixerCommand::RemoveSubTrack(id) => {
				self.sub_tracks.remove(&id);
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
		}
	}

	fn process_track(
		&mut self,
		dt: f64,
		parameters: &Parameters,
		track_index: TrackIndex,
	) -> Frame {
		let mut input = Frame::from_mono(0.0);
		// process all the children of this track and add up their inputs
		for i in 0..self.sub_tracks.len() {
			let (id, track) = self.sub_tracks.get_index(i).unwrap();
			let id = *id;
			if track.parent_track() == track_index {
				input += self.process_track(dt, parameters, TrackIndex::Sub(id));
			}
		}
		// add the cumulative input to this track and run it through the
		// effects chain
		match track_index {
			TrackIndex::Main => {
				self.main_track.add_input(input);
				self.main_track.process(dt, parameters)
			}
			TrackIndex::Sub(id) => {
				if let Some(track) = self.sub_tracks.get_mut(&id) {
					track.add_input(input);
					track.process(dt, parameters)
				} else {
					Frame::from_mono(0.0)
				}
			}
		}
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		self.process_track(dt, parameters, TrackIndex::Main)
	}
}
