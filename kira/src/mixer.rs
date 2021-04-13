use basedrop::Handle;

use crate::Frame;

use self::track::{MainTrack, SubTrack};

pub mod track;

pub(crate) struct Mixer {
	main_track: MainTrack,
	sub_tracks: Vec<SubTrack>,
}

impl Mixer {
	pub fn new(collector_handle: &Handle, sub_track_capacity: usize) -> Self {
		Self {
			main_track: MainTrack::new(collector_handle),
			sub_tracks: Vec::with_capacity(sub_track_capacity),
		}
	}

	pub fn main_track(&self) -> &MainTrack {
		&self.main_track
	}

	pub fn add_sub_track(&mut self, track: SubTrack) {
		self.sub_tracks.push(track);
	}

	pub fn process(&self) -> Frame {
		// we're specifically iterating backwards to make sure
		// the newest tracks are processed first. this way,
		// child tracks should always be processed before their
		// parent tracks, which ensures that tracks are processed
		// in a valid order.
		for sub_track in self.sub_tracks.iter().rev() {
			sub_track.process();
		}
		self.main_track.process()
	}
}
