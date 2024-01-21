use crate::sound::{EndPosition, PlaybackPosition, Region};

use super::Transport;

#[test]
fn stops_at_end() {
	let mut transport = Transport::new(4, None, false, 1, 0);
	for i in 0..4 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position();
	}
	assert_eq!(transport.position, 4);
	assert!(!transport.playing);
}

#[test]
fn stops_at_start_when_playing_backwards() {
	let mut transport = Transport::new(4, None, true, 1, 0);
	for i in (0..4).rev() {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.decrement_position();
	}
	assert_eq!(transport.position, -1);
	assert!(!transport.playing);
}

#[test]
fn start_position() {
	let transport = Transport::new(4, None, false, 1, 3);
	assert_eq!(transport.position, 3);
}

#[test]
fn start_position_reverse() {
	let transport = Transport::new(4, None, true, 1, 2);
	assert_eq!(transport.position, 1);
}

#[test]
fn loops() {
	let mut transport = Transport::new(
		10,
		Some(Region {
			start: PlaybackPosition::Samples(2),
			end: EndPosition::Custom(PlaybackPosition::Samples(5)),
		}),
		false,
		1,
		0,
	);
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
	let mut transport = Transport::new(
		10,
		Some(Region {
			start: PlaybackPosition::Samples(2),
			end: EndPosition::Custom(PlaybackPosition::Samples(5)),
		}),
		true,
		1,
		0,
	);
	for i in (2..10).rev() {
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
	let mut transport = Transport::new(
		10,
		Some(Region {
			start: PlaybackPosition::Samples(2),
			end: EndPosition::Custom(PlaybackPosition::Samples(5)),
		}),
		false,
		1,
		0,
	);
	transport.position = 6;
	transport.increment_position();
	assert_eq!(transport.position, 4);
	transport.position = 1;
	transport.decrement_position();
	assert_eq!(transport.position, 3);
}

#[test]
fn seek_loop_wrapping() {
	let mut transport = Transport::new(
		10,
		Some(Region {
			start: PlaybackPosition::Samples(2),
			end: EndPosition::Custom(PlaybackPosition::Samples(5)),
		}),
		false,
		1,
		0,
	);
	transport.seek_to(7);
	assert_eq!(transport.position, 4);
	transport.seek_to(0);
	assert_eq!(transport.position, 3);
}

#[test]
fn seek_out_of_bounds() {
	let mut transport = Transport::new(10, None, false, 1, 0);
	transport.seek_to(-1);
	assert!(!transport.playing);
	let mut transport = Transport::new(10, None, false, 1, 0);
	transport.seek_to(11);
	assert!(!transport.playing);
}
