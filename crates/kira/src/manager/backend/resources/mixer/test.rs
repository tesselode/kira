use crate::{
	clock::clock_info::MockClockInfoProviderBuilder,
	dsp::Frame,
	modulator::value_provider::MockModulatorValueProviderBuilder,
	track::{SubTrackId, TrackBuilder, TrackRoutes},
};

use super::Mixer;

#[test]
fn parent_routing() {
	let (mut mixer, mut sub_track_controller, _) = Mixer::new(100, 1, TrackBuilder::new());
	let parent_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	sub_track_controller.insert_with_key(
		parent_track_id.0,
		TrackBuilder::new()
			.volume(0.5)
			.build(parent_track_id.into())
			.0,
	);
	let child_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	sub_track_controller.insert_with_key(
		child_track_id.0,
		TrackBuilder::new()
			.routes(TrackRoutes::empty().with_route(parent_track_id, 0.5))
			.build(child_track_id.into())
			.0,
	);
	mixer.on_start_processing();
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
	let (mut mixer, mut sub_track_controller, _) = Mixer::new(100, 1, TrackBuilder::new());
	let send_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	sub_track_controller.insert_with_key(
		send_track_id.0,
		TrackBuilder::new()
			.volume(0.5)
			.build(send_track_id.into())
			.0,
	);
	let other_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	sub_track_controller.insert_with_key(
		other_track_id.0,
		TrackBuilder::new()
			.routes(TrackRoutes::new().with_route(send_track_id, 0.5))
			.build(other_track_id.into())
			.0,
	);
	mixer.on_start_processing();
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
