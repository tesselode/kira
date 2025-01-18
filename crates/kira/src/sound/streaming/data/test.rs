use std::time::Duration;

use crate::{
	frame::Frame,
	sound::streaming::{mock::MockDecoder, StreamingSoundData},
};

#[test]
fn duration() {
	let sound = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.5); 4])),
		settings: Default::default(),
		slice: None,
	};
	assert_eq!(sound.duration(), Duration::from_secs(4));
}

#[test]
fn unsliced_duration() {
	let sound = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.5); 4])),
		settings: Default::default(),
		slice: Some((2, 3)),
	};
	assert_eq!(sound.unsliced_duration(), Duration::from_secs(4));
}

#[test]
fn sliced_duration() {
	let sound = StreamingSoundData {
		decoder: Box::new(MockDecoder::new(vec![Frame::from_mono(0.5); 4])),
		settings: Default::default(),
		slice: Some((2, 3)),
	};
	assert_eq!(sound.duration(), Duration::from_secs(1));
}
