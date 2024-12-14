use crate::{
	track::{MainTrack, MainTrackBuilder, MainTrackHandle},
	Frame,
};

use super::{Clocks, Modulators};

pub struct Mixer {
	main_track: MainTrack,
}

impl Mixer {
	pub fn new(
		main_track_builder: MainTrackBuilder,
	) -> (
		Self,
		// ResourceController<Track>,
		// ResourceController<SendTrack>,
		MainTrackHandle,
	) {
		let (main_track, main_track_handle) = main_track_builder.build();
		(Self { main_track }, main_track_handle)
	}

	pub fn on_start_processing(&mut self) {
		/* self.sub_tracks
			.remove_and_add(|track| track.should_be_removed());
		for (_, track) in &mut self.sub_tracks {
			track.on_start_processing();
		}
		self.send_tracks
			.remove_and_add(|track| track.shared().is_marked_for_removal());
		for (_, track) in &mut self.send_tracks {
			track.on_start_processing();
		} */
		self.main_track.on_start_processing();
	}

	pub(crate) fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
	) {
		self.main_track.process(out, dt, clocks, modulators)
	}
}
