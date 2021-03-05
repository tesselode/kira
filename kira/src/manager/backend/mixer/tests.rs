use basedrop::{Collector, Owned};

use crate::{
	mixer::{SendTrackId, SendTrackSettings, SubTrackId, SubTrackSettings, Track, TrackSends},
	parameter::Parameters,
	Frame,
};

use super::Mixer;

#[test]
fn routes_audio_to_parent_tracks() {
	let collector = Collector::new();
	let parameters = Parameters::new(100);
	let mut mixer = Mixer::new(100, 100);
	// parent track has a volume of 50%
	let parent_track_id = {
		let settings = SubTrackSettings::new().volume(0.5);
		let id = settings.id.unwrap_or(SubTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_sub_track(id, settings),
		));
		id
	};
	// sub tracks 1 and 2 are routed into the parent track
	let sub_track_1_id = {
		let settings = SubTrackSettings::new().parent_track(parent_track_id);
		let id = settings.id.unwrap_or(SubTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_sub_track(id, settings),
		));
		id
	};
	let sub_track_2_id = {
		let settings = SubTrackSettings::new().parent_track(parent_track_id);
		let id = settings.id.unwrap_or(SubTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_sub_track(id, settings),
		));
		id
	};
	// sub track 3 is routed directly to main
	let sub_track_3_id = {
		let settings = SubTrackSettings::new();
		let id = settings.id.unwrap_or(SubTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_sub_track(id, settings),
		));
		id
	};
	// each sub-track will contribute one digit of signal to the final output.
	// sub-tracks 1 and 2 should have their digits be halved to 1,
	// since their parent track has a volume factor of 0.5.
	mixer.add_input(sub_track_1_id.into(), Frame::from_mono(200.0));
	mixer.add_input(sub_track_2_id.into(), Frame::from_mono(020.0));
	mixer.add_input(sub_track_3_id.into(), Frame::from_mono(002.0));
	let out = mixer.process(1.0, &parameters);
	assert_eq!(out, Frame::from_mono(112.0));
}

#[test]
fn routes_audio_to_send_tracks() {
	let collector = Collector::new();
	let parameters = Parameters::new(100);
	let mut mixer = Mixer::new(100, 100);
	let send_track_1_id = {
		let settings = SendTrackSettings::new();
		let id = settings.id.unwrap_or(SendTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_send_track(id, settings),
		));
		id
	};
	let send_track_2_id = {
		let settings = SendTrackSettings::new();
		let id = settings.id.unwrap_or(SendTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_send_track(id, settings),
		));
		id
	};
	let sub_track_id = {
		let settings = SubTrackSettings::new().sends(
			TrackSends::new()
				.add(send_track_1_id, 0.1)
				.add(send_track_2_id, 0.01),
		);
		let id = settings.id.unwrap_or(SubTrackId::new());
		mixer.add_track(Owned::new(
			&collector.handle(),
			Track::new_sub_track(id, settings),
		));
		id
	};
	mixer.add_input(sub_track_id.into(), Frame::from_mono(100.0));
	let out = mixer.process(1.0, &parameters);
	assert_eq!(out, Frame::from_mono(111.0));
}
