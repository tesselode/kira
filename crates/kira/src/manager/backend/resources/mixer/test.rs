use ringbuf::HeapRb;

use crate::{
	clock::clock_info::MockClockInfoProviderBuilder,
	dsp::Frame,
	modulator::value_provider::MockModulatorValueProviderBuilder,
	track::{SubTrackId, TrackBuilder, TrackRoutes},
};

use super::Mixer;

#[test]
fn parent_routing() {
	let (unused_sub_track_producer, _) = HeapRb::new(1).split();
	let (mut mixer, _) = Mixer::new(100, unused_sub_track_producer, 1, TrackBuilder::new());
	let sub_track_controller = mixer.sub_track_controller();
	let parent_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.add_sub_track(
		parent_track_id,
		TrackBuilder::new()
			.volume(0.5)
			.build(parent_track_id.into())
			.0,
	);
	let child_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.add_sub_track(
		child_track_id,
		TrackBuilder::new()
			.routes(TrackRoutes::empty().with_route(parent_track_id, 0.5))
			.build(parent_track_id.into())
			.0,
	);
	mixer
		.track_mut(child_track_id.into())
		.unwrap()
		.add_input(Frame::from_mono(1.0));
	assert_eq!(
		mixer.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(0.25)
	);
}

#[test]
fn send_routing() {
	let (unused_sub_track_producer, _) = HeapRb::new(1).split();
	let (mut mixer, _) = Mixer::new(100, unused_sub_track_producer, 1, TrackBuilder::new());
	let sub_track_controller = mixer.sub_track_controller();
	let send_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.add_sub_track(
		send_track_id,
		TrackBuilder::new()
			.volume(0.5)
			.build(send_track_id.into())
			.0,
	);
	let other_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.add_sub_track(
		other_track_id,
		TrackBuilder::new()
			.routes(TrackRoutes::new().with_route(send_track_id, 0.5))
			.build(other_track_id.into())
			.0,
	);
	mixer
		.track_mut(other_track_id.into())
		.unwrap()
		.add_input(Frame::from_mono(1.0));
	assert_eq!(
		mixer.process(
			1.0,
			&MockClockInfoProviderBuilder::new(0).build(),
			&MockModulatorValueProviderBuilder::new(0).build()
		),
		Frame::from_mono(1.25)
	);
}
