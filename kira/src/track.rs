mod effect;
mod handle;
mod routes;
mod settings;

pub use effect::*;
pub use handle::*;
pub use routes::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::Index;

use crate::{
	frame::Frame,
	manager::{context::Context, resources::Parameters},
	value::{cached::CachedValue, Value},
};

/// A unique identifier for a mixer sub-track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubTrackId(pub(crate) Index);

/// A unique identifier for a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackId {
	/// The main mixer track.
	Main,
	/// A sub-track.
	Sub(SubTrackId),
}

impl From<SubTrackId> for TrackId {
	fn from(id: SubTrackId) -> Self {
		Self::Sub(id)
	}
}

impl From<&TrackHandle> for TrackId {
	fn from(handle: &TrackHandle) -> Self {
		handle.id()
	}
}

pub(crate) struct TrackShared {
	removed: AtomicBool,
}

impl TrackShared {
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

pub(crate) struct Track {
	shared: Arc<TrackShared>,
	volume: CachedValue,
	panning: CachedValue,
	routes: Vec<(TrackId, CachedValue)>,
	effects: Vec<Box<dyn Effect>>,
	input: Frame,
}

impl Track {
	pub fn new(mut settings: TrackSettings, context: &Arc<Context>) -> Self {
		for effect in &mut settings.effects {
			effect.init(context.sample_rate());
		}
		Self {
			shared: Arc::new(TrackShared::new()),
			volume: CachedValue::new(.., settings.volume, 1.0),
			panning: CachedValue::new(0.0..=1.0, settings.panning, 0.5),
			routes: settings.routes.into_vec(),
			effects: settings.effects,
			input: Frame::from_mono(0.0),
		}
	}

	pub fn shared(&self) -> Arc<TrackShared> {
		self.shared.clone()
	}

	pub fn routes_mut(&mut self) -> &mut Vec<(TrackId, CachedValue)> {
		&mut self.routes
	}

	pub fn set_volume(&mut self, volume: Value) {
		self.volume.set(volume);
	}

	pub fn set_panning(&mut self, panning: Value) {
		self.panning.set(panning);
	}

	pub fn add_input(&mut self, input: Frame) {
		self.input += input;
	}

	pub fn process(&mut self, dt: f64, parameters: &Parameters) -> Frame {
		self.volume.update(parameters);
		self.panning.update(parameters);
		for (_, amount) in &mut self.routes {
			amount.update(parameters);
		}
		let mut output = std::mem::replace(&mut self.input, Frame::from_mono(0.0));
		for effect in &mut self.effects {
			output = effect.process(output, dt, parameters);
		}
		output *= self.volume.get() as f32;
		output = output.panned(self.panning.get() as f32);
		output
	}
}
