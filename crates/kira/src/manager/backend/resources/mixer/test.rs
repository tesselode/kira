use ringbuf::RingBuffer;

use crate::{
	dsp::Frame,
	manager::command::MixerCommand,
	track::{SubTrackId, Track, TrackBuilder, TrackRoutes},
};

use super::Mixer;

#[test]
fn parent_routing() {
	let (unused_sub_track_producer, _) = RingBuffer::new(1).split();
	let mut mixer = Mixer::new(100, unused_sub_track_producer, 1, TrackBuilder::new());
	let sub_track_controller = mixer.sub_track_controller();
	let parent_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.run_command(MixerCommand::AddSubTrack(
		parent_track_id,
		Track::new(TrackBuilder::new().volume(0.5)),
	));
	let child_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.run_command(MixerCommand::AddSubTrack(
		child_track_id,
		Track::new(
			TrackBuilder::new().routes(TrackRoutes::empty().with_route(parent_track_id, 0.5)),
		),
	));
	mixer
		.track_mut(child_track_id.into())
		.unwrap()
		.add_input(Frame::from_mono(1.0));
	assert_eq!(mixer.process(1.0), Frame::from_mono(0.25));
}

#[test]
fn send_routing() {
	let (unused_sub_track_producer, _) = RingBuffer::new(1).split();
	let mut mixer = Mixer::new(100, unused_sub_track_producer, 1, TrackBuilder::new());
	let sub_track_controller = mixer.sub_track_controller();
	let send_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.run_command(MixerCommand::AddSubTrack(
		send_track_id,
		Track::new(TrackBuilder::new().volume(0.5)),
	));
	let other_track_id = SubTrackId(sub_track_controller.try_reserve().unwrap());
	mixer.run_command(MixerCommand::AddSubTrack(
		other_track_id,
		Track::new(TrackBuilder::new().routes(TrackRoutes::new().with_route(send_track_id, 0.5))),
	));
	mixer
		.track_mut(other_track_id.into())
		.unwrap()
		.add_input(Frame::from_mono(1.0));
	assert_eq!(mixer.process(1.0), Frame::from_mono(1.25));
}
