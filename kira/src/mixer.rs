pub mod effect;
pub mod effect_slot;
pub mod track;

use basedrop::Owned;
use ringbuf::RingBuffer;

use crate::{value::Value, Frame};

use self::track::Track;

pub(crate) struct Mixer {
	main_track: Track,
	sub_tracks: Vec<Owned<Track>>,
}

impl Mixer {
	pub fn new(sub_track_capacity: usize) -> Self {
		// TODO: expose a way to add effects to the main track
		let (effect_slot_producer, effect_slot_consumer) = RingBuffer::new(0).split();
		Self {
			main_track: Track::new(vec![], Value::Fixed(1.0), 0, effect_slot_consumer),
			sub_tracks: Vec::with_capacity(sub_track_capacity),
		}
	}

	pub fn main_track(&self) -> &Track {
		&self.main_track
	}

	pub fn add_sub_track(&mut self, track: Owned<Track>) {
		self.sub_tracks.push(track);
	}

	pub fn process(&mut self, dt: f64) -> Frame {
		// we're specifically iterating backwards to make sure
		// the newest tracks are processed first. this way,
		// child tracks should always be processed before their
		// parent tracks, which ensures that tracks are processed
		// in a valid order.
		for sub_track in self.sub_tracks.iter_mut().rev() {
			sub_track.process(dt);
		}
		self.main_track.process(dt)
	}
}
