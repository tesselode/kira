use std::time::Duration;

use kira::{
	backend::mock::MockBackend,
	manager::{AudioManager, AudioManagerSettings},
	sound::{
		streaming::{Decoder, StreamingSoundData},
		PlaybackState,
	},
	Frame,
};

struct MockDecoder;

#[derive(Debug, PartialEq, Eq)]
struct MockDecoderError;

impl Decoder for MockDecoder {
	type Error = MockDecoderError;

	fn sample_rate(&self) -> u32 {
		1
	}

	fn num_frames(&self) -> usize {
		1
	}

	fn decode(&mut self) -> Result<Vec<Frame>, Self::Error> {
		Err(MockDecoderError)
	}

	fn seek(&mut self, _index: usize) -> Result<usize, Self::Error> {
		Ok(0)
	}
}

#[test]
fn streaming_sound_stops_on_error() {
	let mut manager = AudioManager::<MockBackend>::new(AudioManagerSettings::default()).unwrap();
	let data = StreamingSoundData::from_decoder(MockDecoder);
	let mut sound = manager.play(data).unwrap();
	manager.backend_mut().on_start_processing();
	std::thread::sleep(Duration::from_secs(1));
	manager.backend_mut().process();
	manager.backend_mut().on_start_processing();
	assert_eq!(sound.state(), PlaybackState::Stopped);
	assert_eq!(sound.pop_error(), Some(MockDecoderError));
	assert_eq!(manager.main_track().num_sounds(), 0);
}
