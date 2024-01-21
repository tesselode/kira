use super::Transport;

#[test]
fn stops_at_end() {
	let mut transport = Transport {
		position: 2,
		loop_region: None,
		playing: true,
	};
	for i in 2..4 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position(4);
	}
	assert_eq!(transport.position, 4);
	assert!(!transport.playing);
}

#[test]
fn stops_at_start_when_playing_backwards() {
	let mut transport = Transport {
		position: 2,
		loop_region: None,
		playing: true,
	};
	for i in (0..=2).rev() {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.decrement_position();
	}
	assert_eq!(transport.position, 0);
	assert!(!transport.playing);
}

#[test]
fn loops() {
	let mut transport = Transport {
		position: 0,
		loop_region: Some((2, 5)),
		playing: true,
	};
	for i in 0..5 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position(10);
	}
	for i in 2..5 {
		assert_eq!(transport.position, i);
		assert!(transport.playing);
		transport.increment_position(10);
	}
}

#[test]
fn loops_when_playing_backward() {
	let mut transport = Transport {
		position: 0,
		loop_region: Some((2, 5)),
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
		loop_region: Some((2, 5)),
		playing: true,
	};
	transport.position = 6;
	transport.increment_position(10);
	assert_eq!(transport.position, 4);
	transport.position = 1;
	transport.decrement_position();
	assert_eq!(transport.position, 3);
}

#[test]
fn seek_loop_wrapping() {
	let mut transport = Transport {
		position: 0,
		loop_region: Some((2, 5)),
		playing: true,
	};
	transport.seek_to(7, 10);
	assert_eq!(transport.position, 4);
	transport.seek_to(0, 10);
	assert_eq!(transport.position, 3);
}

#[test]
fn seek_out_of_bounds() {
	let mut transport = Transport::new(5, None, false, 1, 10);
	transport.seek_to(10, 10);
	assert!(!transport.playing);
}
