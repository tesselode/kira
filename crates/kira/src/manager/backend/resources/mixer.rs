use crate::{
	clock::clock_info::ClockInfoProvider,
	frame::Frame,
	listener::ListenerInfoProvider,
	modulator::value_provider::ModulatorValueProvider,
	track::{MainTrack, MainTrackBuilder, MainTrackHandle, SendTrack, Track},
};

use super::{ResourceController, ResourceStorage};

pub(crate) struct Mixer {
	main_track: MainTrack,
	sub_tracks: ResourceStorage<Track>,
	send_tracks: ResourceStorage<SendTrack>,
}

impl Mixer {
	#[must_use]
	pub fn new(
		sub_track_capacity: u16,
		send_track_capacity: u16,
		sample_rate: u32,
		main_track_builder: MainTrackBuilder,
	) -> (
		Self,
		ResourceController<Track>,
		ResourceController<SendTrack>,
		MainTrackHandle,
	) {
		let (mut main_track, main_track_handle) = main_track_builder.build();
		main_track.init_effects(sample_rate);
		let (sub_tracks, sub_track_controller) = ResourceStorage::new(sub_track_capacity);
		let (send_tracks, send_track_controller) = ResourceStorage::new(send_track_capacity);
		(
			Self {
				main_track,
				sub_tracks,
				send_tracks,
			},
			sub_track_controller,
			send_track_controller,
			main_track_handle,
		)
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.main_track.on_change_sample_rate(sample_rate);
		for (_, track) in &mut self.sub_tracks {
			track.on_change_sample_rate(sample_rate);
		}
		for (_, track) in &mut self.send_tracks {
			track.on_change_sample_rate(sample_rate);
		}
	}

	pub fn on_start_processing(&mut self) {
		self.sub_tracks
			.remove_and_add(|track| track.should_be_removed());
		for (_, track) in &mut self.sub_tracks {
			track.on_start_processing();
		}
		self.send_tracks
			.remove_and_add(|track| track.shared().is_marked_for_removal());
		for (_, track) in &mut self.send_tracks {
			track.on_start_processing();
		}
		self.main_track.on_start_processing();
	}

	#[must_use]
	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
		listener_info_provider: &ListenerInfoProvider,
	) -> Frame {
		let mut main_track_input = Frame::ZERO;
		for (_, track) in &mut self.sub_tracks {
			main_track_input += track.process(
				dt,
				clock_info_provider,
				modulator_value_provider,
				listener_info_provider,
				&mut self.send_tracks,
			);
		}
		for (_, track) in &mut self.send_tracks {
			main_track_input += track.process(dt, clock_info_provider, modulator_value_provider);
		}
		self.main_track.process(
			main_track_input,
			dt,
			clock_info_provider,
			modulator_value_provider,
		)
	}
}
