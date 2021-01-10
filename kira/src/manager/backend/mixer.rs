use flume::Sender;
use indexmap::IndexMap;

use crate::{
	command::MixerCommand,
	frame::Frame,
	mixer::{SubTrackId, Track, TrackIndex, TrackSettings},
	parameter::Parameters,
	resource::Resource,
};

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: IndexMap<SubTrackId, Track>,
}

impl Mixer {
	pub fn new() -> Self {
		Self {
			main_track: Track::new(TrackSettings::default()),
			sub_tracks: IndexMap::new(),
		}
	}

	pub fn run_command(&mut self, command: MixerCommand, unloader: &mut Sender<Resource>) {
		match command {
			MixerCommand::AddSubTrack(track) => {
				if let Some(track) = self.sub_tracks.insert(track.id(), track) {
					unloader.try_send(Resource::Track(track)).ok();
				}
			}
			MixerCommand::AddEffect(index, effect, settings) => {
				let track = match index {
					TrackIndex::Main => Some(&mut self.main_track),
					TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id),
				};
				if let Some(track) = track {
					track.add_effect(effect, settings);
				}
			}
			MixerCommand::RemoveSubTrack(id) => {
				if let Some(track) = self.sub_tracks.remove(&id) {
					unloader.try_send(Resource::Track(track)).ok();
				}
			}
			MixerCommand::SetEffectEnabled(track_index, effect_id, enabled) => {
				let track = match track_index {
					TrackIndex::Main => Some(&mut self.main_track),
					TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id),
				};
				if let Some(track) = track {
					if let Some(effect_slot) = track.effect_mut(effect_id) {
						effect_slot.enabled = enabled;
					}
				}
			}
			MixerCommand::RemoveEffect(track_index, effect_id) => {
				let track = match track_index {
					TrackIndex::Main => Some(&mut self.main_track),
					TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id),
				};
				if let Some(track) = track {
					if let Some(effect_slot) = track.remove_effect(effect_id) {
						unloader.try_send(Resource::EffectSlot(effect_slot)).ok();
					}
				}
			}
		}
	}

	pub fn track_mut(&mut self, index: TrackIndex) -> Option<&mut Track> {
		match index {
			TrackIndex::Main => Some(&mut self.main_track),
			TrackIndex::Sub(id) => self.sub_tracks.get_mut(&id),
		}
	}

	pub fn add_input(&mut self, index: TrackIndex, input: Frame) {
		if let Some(track) = self.track_mut(index) {
			track.add_input(input);
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
		if let Some(track) = self.track_mut(track_index) {
			track.add_input(input);
			track.process(dt, parameters)
		} else {
			Frame::from_mono(0.0)
		}
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		self.process_track(dt, parameters, TrackIndex::Main)
	}
}
