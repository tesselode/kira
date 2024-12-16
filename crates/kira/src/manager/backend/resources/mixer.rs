use crate::{
	frame::Frame,
	info::Info,
	track::{MainTrack, MainTrackBuilder, MainTrackHandle, SendTrack, Track},
};

use super::{
	clocks::Clocks, listeners::Listeners, modulators::Modulators, ResourceController,
	ResourceStorage,
};

pub(crate) struct Mixer {
	main_track: MainTrack,
	sub_tracks: ResourceStorage<Track>,
	send_tracks: ResourceStorage<SendTrack>,
	temp_buffer: Vec<Frame>,
}

impl Mixer {
	#[must_use]
	pub fn new(
		sub_track_capacity: u16,
		send_track_capacity: u16,
		sample_rate: u32,
		internal_buffer_size: usize,
		main_track_builder: MainTrackBuilder,
	) -> (
		Self,
		ResourceController<Track>,
		ResourceController<SendTrack>,
		MainTrackHandle,
	) {
		let (mut main_track, main_track_handle) = main_track_builder.build(internal_buffer_size);
		main_track.init_effects(sample_rate);
		let (sub_tracks, sub_track_controller) = ResourceStorage::new(sub_track_capacity);
		let (send_tracks, send_track_controller) = ResourceStorage::new(send_track_capacity);
		(
			Self {
				main_track,
				sub_tracks,
				send_tracks,
				temp_buffer: vec![Frame::ZERO; internal_buffer_size],
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

	pub fn process(
		&mut self,
		out: &mut [Frame],
		dt: f64,
		clocks: &Clocks,
		modulators: &Modulators,
		listeners: &Listeners,
	) {
		for (_, track) in &mut self.sub_tracks {
			track.process(
				&mut self.temp_buffer[..out.len()],
				dt,
				clocks,
				modulators,
				listeners,
				None,
				&mut self.send_tracks,
			);
			for (summed_out, sound_out) in out.iter_mut().zip(self.temp_buffer.iter().copied()) {
				*summed_out += sound_out;
			}
			self.temp_buffer.fill(Frame::ZERO);
		}
		let info = Info::new(
			&clocks.0.resources,
			&modulators.0.resources,
			&listeners.0.resources,
			None,
		);
		for (_, track) in &mut self.send_tracks {
			track.process(&mut self.temp_buffer[..out.len()], dt, &info);
			for (summed_out, sound_out) in out.iter_mut().zip(self.temp_buffer.iter().copied()) {
				*summed_out += sound_out;
			}
			self.temp_buffer.fill(Frame::ZERO);
		}
		self.main_track.process(out, dt, &info);
	}
}
