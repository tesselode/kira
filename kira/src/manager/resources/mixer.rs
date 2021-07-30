use atomic_arena::{Arena, Controller};
use ringbuf::Producer;

use crate::{
	frame::Frame,
	track::{SubTrackId, Track, TrackId},
	value::cached::CachedValue,
};

use super::parameters::Parameters;

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: Arena<Track>,
	sub_track_ids: Vec<SubTrackId>,
	dummy_routes: Vec<(TrackId, CachedValue)>,
	unused_track_producer: Producer<Track>,
}

impl Mixer {
	pub fn new(sub_track_capacity: usize, unused_track_producer: Producer<Track>) -> Self {
		Self {
			main_track: Track::new(Default::default()),
			sub_tracks: Arena::new(sub_track_capacity),
			sub_track_ids: Vec::with_capacity(sub_track_capacity),
			dummy_routes: vec![],
			unused_track_producer,
		}
	}

	pub fn sub_tracks_controller(&self) -> Controller {
		self.sub_tracks.controller()
	}

	pub fn process(&mut self, parameters: &Parameters) -> Frame {
		// iterate through the sub-tracks newest to oldest
		for id in self.sub_track_ids.iter().rev() {
			// process the track and get its output
			let track = self
				.sub_tracks
				.get_mut(id.0)
				.expect("sub track IDs and sub tracks are out of sync");
			let output = track.process(parameters);
			// temporarily take ownership of its routes. we can't just
			// borrow the routes because then we can't get mutable
			// references to the other tracks
			std::mem::swap(track.routes_mut(), &mut self.dummy_routes);
			// send the output to the destination tracks
			for (id, amount) in &self.dummy_routes {
				let send_track = match id {
					TrackId::Main => Some(&mut self.main_track),
					TrackId::Sub(id) => self.sub_tracks.get_mut(id.0),
				};
				if let Some(send_track) = send_track {
					send_track.add_input(output * amount.get() as f32);
				}
			}
			// borrow the track again and give it back its routes
			let track = self
				.sub_tracks
				.get_mut(id.0)
				.expect("sub track IDs and sub tracks are out of sync");
			std::mem::swap(track.routes_mut(), &mut self.dummy_routes);
		}
		self.main_track.process(parameters)
	}
}
