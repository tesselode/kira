use crate::sound::{EndPosition, PlaybackPosition, Region};
use super::Transport;

#[test]
fn stops_at_end() {
	let mut transport = Transport {
		position: 2,
		playback_region: (2, Some(4)),
		loop_region: None,
		reverse: false,
		playing: true,
	};
	for i in 2..=4 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position();
	}
	assert_eq!(transport.position, 5);
	assert!(!transport.playing);
}

#[test]
fn stops_at_start_when_playing_backwards() {
	let mut transport = Transport {
		position: 2,
		playback_region: (2, Some(4)),
		loop_region: None,
		reverse: false,
		playing: true,
	};
	transport.position = 4;
	for i in (2..=4).rev() {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.decrement_position();
	}
	assert_eq!(transport.position, 1);
	assert!(!transport.playing);
}

#[test]
fn loops() {
	let mut transport = Transport {
		position: 0,
		playback_region: (0, Some(10)),
		loop_region: Some((2, 5)),
		reverse: false,
		playing: true,
	};
	for i in 0..5 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position();
	}
	for i in 2..5 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position();
	}
}

#[test]
fn loops_when_playing_backward() {
	let mut transport = Transport {
		position: 0,
		playback_region: (0, Some(10)),
		loop_region: Some((2, 5)),
		reverse: false,
		playing: true,
	};
	transport.position = 10;
	for i in (2..=10).rev() {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.decrement_position();
	}
	for i in (2..5).rev() {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.decrement_position();
	}
}

#[test]
fn loop_wrapping() {
	let mut transport = Transport {
		position: 0,
		playback_region: (0, Some(10)),
		loop_region: Some((2, 5)),
		reverse: false,
		playing: true,
	};
	transport.position = 6;
	transport.increment_position();
	assert_eq!(transport.position, 4);
	transport.position = 1;
	transport.decrement_position();
	assert_eq!(transport.position, 3);
}

#[test]
fn seek_loop_wrapping() {
	let mut transport = Transport {
		position: 0,
		playback_region: (0, Some(10)),
		loop_region: Some((2, 5)),
		reverse: false,
		playing: true,
	};
	transport.seek_to(7);
	assert_eq!(transport.position, 4);
	transport.seek_to(0);
	assert_eq!(transport.position, 3);
}

#[test]
fn seek_out_of_bounds() {
	let mut transport = Transport {
		position: 0,
		playback_region: (0, Some(10)),
		loop_region: None,
		reverse: false,
		playing: true,
	};
	transport.seek_to(-1);
	assert!(!transport.playing);
	let mut transport = Transport {
		position: 0,
		playback_region: (0, Some(10)),
		loop_region: None,
		reverse: false,
		playing: true,
	};
	transport.seek_to(11);
	assert!(!transport.playing);
}

#[test]
fn seek_no_upper_bounds_without_playback_end() {
	let mut transport = Transport {
		position: 0,
		playback_region: (0, None),
		loop_region: None,
		reverse: false,
		playing: true,
	};
	transport.seek_to(-1);
	assert!(!transport.playing);

	let mut transport = Transport {
		position: 0,
		playback_region: (0, None),
		loop_region: None,
		reverse: false,
		playing: true,
	};
	transport.seek_to(i64::MAX);
	assert!(transport.playing);
}

#[test]
fn prevents_playing_backwards_without_playback_end() {
	std::panic::set_hook(Box::new(|_| {}));

	let playback_region_end = Region {
		start: PlaybackPosition::Samples(0),
		end: EndPosition::EndOfAudio
	};

	assert!(std::panic::catch_unwind(|| Transport::new(playback_region_end, None,false, 0, None)).is_ok());
	assert!(std::panic::catch_unwind(|| Transport::new(playback_region_end, None,true, 0, None)).is_err());

	let playback_region_position = Region {
		start: PlaybackPosition::Samples(0),
		end: EndPosition::Custom(PlaybackPosition::Samples(10))
	};

	assert!(std::panic::catch_unwind(|| Transport::new(playback_region_position, None,true, 0, None)).is_ok());

	let _hook = std::panic::take_hook();
}