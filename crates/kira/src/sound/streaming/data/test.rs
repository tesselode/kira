use std::time::Duration;

use crate::{
	dsp::Frame,
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
