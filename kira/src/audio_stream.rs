//! Continuous, infinite streams of audio.

mod handle;

pub use handle::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Index;

use crate::{
	manager::{
		context::Context,
		resources::{mixer::Mixer, Parameters},
	},
	track::TrackId,
	Frame,
};

/// A unique identifier for an audio stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioStreamId(pub(crate) Index);

/// Produces a continuous stream of audio data.
#[allow(unused_variables)]
pub trait AudioStream: Send + Sync {
	/// Called when the [`AudioStream`] is first sent to the renderer.
	fn init(&mut self, sample_rate: u32) {}

	/// Called when the [`AudioStream`] should produce a new [`Frame`]
	/// of audio.
	/// - `dt` is the time that's elapsed since the previous round of
	/// processing (in seconds)
	/// - `parameters` contains information about the current value of
	/// parameters. This is an opaque type that's only useful for updating
	/// `CachedValue`s.
	fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame;
}

pub(crate) struct AudioStreamShared {
	removed: AtomicBool,
}

impl AudioStreamShared {
	pub fn new() -> Self {
		Self {
			removed: AtomicBool::new(false),
		}
	}

	pub fn is_marked_for_removal(&self) -> bool {
		self.removed.load(Ordering::SeqCst)
	}

	pub fn mark_for_removal(&self) {
		self.removed.store(true, Ordering::SeqCst);
	}
}

pub(crate) struct AudioStreamWrapper {
	shared: Arc<AudioStreamShared>,
	stream: Box<dyn AudioStream>,
	track_id: TrackId,
}

impl AudioStreamWrapper {
	pub fn new(
		mut stream: Box<dyn AudioStream>,
		track_id: TrackId,
		context: &Arc<Context>,
	) -> Self {
		stream.init(context.sample_rate());
		Self {
			shared: Arc::new(AudioStreamShared::new()),
			stream,
			track_id,
		}
	}

	pub fn shared(&self) -> Arc<AudioStreamShared> {
		self.shared.clone()
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters, mixer: &mut Mixer) {
		if let Some(track) = mixer.track_mut(self.track_id) {
			track.add_input(self.stream.process(dt, parameters));
		}
	}
}
