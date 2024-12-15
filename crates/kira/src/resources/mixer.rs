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
		sample_rate: u32,
		main_track_builder: MainTrackBuilder,
	) -> (
		Self,
		// ResourceController<Track>,
		// ResourceController<SendTrack>,
		MainTrackHandle,
	) {
		let (mut main_track, main_track_handle) = main_track_builder.build();
		main_track.init_effects(sample_rate);
		(Self { main_track }, main_track_handle)
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.main_track.on_change_sample_rate(sample_rate);
		/* for (_, track) in &mut self.sub_tracks {
			track.on_change_sample_rate(sample_rate);
		}
		for (_, track) in &mut self.send_tracks {
			track.on_change_sample_rate(sample_rate);
		} */
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
