#[cfg(test)]
mod test;

use crate::{
	clock::clock_info::ClockInfoProvider,
	dsp::Frame,
	modulator::value_provider::ModulatorValueProvider,
	track::{Track, TrackBuilder, TrackHandle, TrackId},
};

use super::{ResourceController, SelfReferentialResourceStorage};

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: SelfReferentialResourceStorage<Track>,
}

impl Mixer {
	pub fn new(
		sub_track_capacity: u16,
		sample_rate: u32,
		main_track_builder: TrackBuilder,
	) -> (Self, ResourceController<Track>, TrackHandle) {
		let (mut main_track, main_track_handle) = main_track_builder.build(TrackId::Main);
		main_track.init_effects(sample_rate);
		let (sub_tracks, sub_track_controller) =
			SelfReferentialResourceStorage::new(sub_track_capacity);
		(
			Self {
				main_track,
				sub_tracks,
			},
			sub_track_controller,
			main_track_handle,
		)
	}

	pub fn track_mut(&mut self, id: TrackId) -> Option<&mut Track> {
		match id {
			TrackId::Main => Some(&mut self.main_track),
			TrackId::Sub(id) => self.sub_tracks.get_mut(id.0),
		}
	}

	pub fn on_change_sample_rate(&mut self, sample_rate: u32) {
		self.main_track.on_change_sample_rate(sample_rate);
		for (_, track) in &mut self.sub_tracks {
			track.on_change_sample_rate(sample_rate);
		}
	}

	pub fn on_start_processing(&mut self) {
		self.sub_tracks
			.remove_and_add(|track| track.shared().is_marked_for_removal());
		for (_, track) in &mut self.sub_tracks {
			track.on_start_processing();
		}
		self.main_track.on_start_processing();
	}

	pub fn process(
		&mut self,
		dt: f64,
		clock_info_provider: &ClockInfoProvider,
		modulator_value_provider: &ModulatorValueProvider,
	) -> Frame {
		self.sub_tracks.for_each_rev(|track, others| {
			let output = track.process(dt, clock_info_provider, modulator_value_provider);
			for (id, route) in track.routes_mut() {
				let destination_track = match id {
					TrackId::Main => Some(&mut self.main_track),
					TrackId::Sub(id) => others.get_mut(id.0),
				};
				if let Some(destination_track) = destination_track {
					destination_track
						.add_input(output * route.volume.value().as_amplitude() as f32);
				}
			}
		});
		self.main_track
			.process(dt, clock_info_provider, modulator_value_provider)
	}
}
