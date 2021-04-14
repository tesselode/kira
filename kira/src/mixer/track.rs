pub mod handle;
pub mod settings;

use atomig::{Atomic, Ordering};
use basedrop::{Handle, Shared};

use crate::{value::Value, Frame};

#[derive(Clone)]
pub(crate) struct TrackInput(Shared<Atomic<Frame>>);

impl TrackInput {
	fn new(collector_handle: &Handle) -> Self {
		Self(Shared::new(
			collector_handle,
			Atomic::new(Frame::from_mono(0.0)),
		))
	}

	pub(crate) fn add(&self, frame: Frame) {
		let previous = self.0.load(Ordering::SeqCst);
		self.0.store(previous + frame, Ordering::SeqCst);
	}

	fn take(&self) -> Frame {
		self.0.swap(Frame::from_mono(0.0), Ordering::SeqCst)
	}
}

pub(crate) struct Track {
	input: TrackInput,
	output_dest: Option<TrackInput>,
	volume: Value<f64>,
}

impl Track {
	pub fn new(
		collector_handle: &Handle,
		output_dest: Option<TrackInput>,
		volume: Value<f64>,
	) -> Self {
		Self {
			input: TrackInput::new(collector_handle),
			output_dest,
			volume,
		}
	}

	pub fn input(&self) -> &TrackInput {
		&self.input
	}

	pub fn process(&self) -> Frame {
		let out = self.input.take() * self.volume.get() as f32;
		if let Some(output_dest) = &self.output_dest {
			output_dest.add(out);
		}
		out
	}
}
