mod handle;
mod settings;

pub use handle::*;
pub use settings::*;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use atomic_arena::{Arena, Key};

use crate::{dsp::Frame, math::Vec3, track::TrackId, tween::Tweenable, Volume};

use super::{emitter::Emitter, scene::SpatialSceneId};

pub(crate) struct Listener {
	shared: Arc<ListenerShared>,
	position: Vec3,
	track: TrackId,
}

impl Listener {
	pub fn new(settings: ListenerSettings) -> Self {
		Self {
			shared: Arc::new(ListenerShared::new()),
			position: settings.position,
			track: settings.track,
		}
	}

	pub fn shared(&self) -> Arc<ListenerShared> {
		self.shared.clone()
	}

	pub fn track(&self) -> TrackId {
		self.track
	}

	pub fn set_position(&mut self, position: Vec3) {
		self.position = position;
	}

	pub fn process(&mut self, emitters: &Arena<Emitter>) -> Frame {
		let mut output = Frame::ZERO;
		for (_, emitter) in emitters {
			let mut emitter_output = emitter.output();
			if let Some(attenuation_function) = emitter.attenuation_function() {
				let distance = (emitter.position() - self.position).magnitude();
				let relative_distance = emitter.distances().relative_distance(distance);
				let relative_volume =
					attenuation_function.apply((1.0 - relative_distance).into()) as f32;
				let amplitude = Tweenable::lerp(
					Volume::Decibels(Volume::MIN_DECIBELS),
					Volume::Decibels(0.0),
					relative_volume.into(),
				)
				.as_amplitude() as f32;
				emitter_output *= amplitude;
			}
			output += emitter_output;
		}
		output
	}
}

/// A unique identifier for an listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId {
	pub(crate) key: Key,
	pub(crate) scene_id: SpatialSceneId,
}

impl ListenerId {
	/// Returns the ID of the spatial scene this listener belongs to.
	pub fn scene(&self) -> SpatialSceneId {
		self.scene_id
	}
}

pub(crate) struct ListenerShared {
	removed: AtomicBool,
}

impl ListenerShared {
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
